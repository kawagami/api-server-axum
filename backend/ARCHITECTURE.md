# kawa-web backend — 架構與維運筆記

kawa-web monorepo 的後端（Rust 2021，async/tokio），生產站台 `https://api.kawa.homes`（舊名 `axum.kawa.homes` 保留為 nginx alias，接 repo 外殘留的舊連結）。

> 功能清單、API 路由總表、排程 job 表、技術棧、環境變數見同目錄 `README.md`；本檔是內部架構、不變式、踩過的坑與已知技術債。
> 引用 `docs/**` 的連結指向**不進版控**的本機文件，只當備查；要長期成立的結論一律寫在本檔本文。

## 架構

```
src/
├── main.rs            # 入口：初始化 tracing、綁定 TCP、優雅關閉（5 秒排水上限，見「優雅關閉」）
├── routes.rs          # app() — 合併所有 Router、CORS、request timeout(60s)、request span + access log、catch panic、錯誤形狀正規化、注入 state、啟動 scheduler
├── extract.rs         # 全站 extractor（Json / Query / Path）—— rejection 轉 AppError；clippy.toml 禁用 axum 原生版
├── batch_writer.rs    # logs 與 admin_audit_logs 共用的批次寫入迴圈（常數只有一份）
├── state.rs           # AppState(Arc<AppStateInner>) — 跨 handler 共用狀態
├── scheduler.rs       # tokio-cron-scheduler — 啟動時註冊所有 job；上一輪未完跳過本輪（per-job try_lock）
├── errors.rs          # 錯誤型別
├── logging.rs         # tracing 初始化 + DbLogLayer（WARN/ERROR 落 logs 表）、log_writer、static DB_LEVEL / ACCESS_TARGET；見「應用日誌」
├── games/             # 對戰遊戲：共用框架 + 各遊戲純引擎（零 WS / 零 IO，可單測；測試住 `<game>/engine/tests.rs` 子模組）
│   ├── common/        # 泛型框架：engine.rs（GameEngine trait + Side/GameStatus）、hub.rs（HubInner<E>/Table/Game）、service.rs（大廳/桌位/配對/行棋/Fischer 計時/斷線，全泛型）、room.rs（RoomKind trait + N 人房大廳/房間 CRUD/斷線，avalon/farm 共用）
│   ├── chess/         # 象棋：engine.rs（走法/將軍/將死/飛將/困斃/60回合和）、types.rs、game.rs（impl GameEngine）
│   ├── gomoku/        # 五子棋：engine.rs（15×15 五連勝）、game.rs
│   ├── banqi/         # 暗棋翻棋：engine.rs（4×8 翻子定色/位階吃子/炮隔架/卒吃將）、game.rs
│   ├── western_chess/ # 西洋棋：engine.rs（走法/王車易位/吃過路兵/升變/將死/逼和/50步和/子力不足和）、types.rs、game.rs
│   ├── go/            # 圍棋：engine.rs（19×19 提子/禁自殺/簡單劫/雙虛手數子+komi 7.5）、game.rs
│   ├── avalon/        # 阿瓦隆（N 人社交推理，非 GameEngine）：roles.rs、engine.rs（組隊/投票/任務/刺客）、hub.rs（impl RoomKind）、service.rs（私有角色推送/階段機/chat；大廳/房間走 common::room）
│   ├── farm/          # 農場經營（N 人 worker-placement，完全資訊）：engine.rs（行動格/收穫/餵食/繁殖/計分）、hub.rs（impl RoomKind）、service.rs（全盤狀態廣播；大廳/房間走 common::room）
│   └── registry.rs    # AnyHub enum 收斂所有遊戲 hub（含非 GameEngine 的 Avalon / Farm）
├── middleware/        # Axum middleware
│   ├── auth.rs        # JWT 驗證 middleware（authorize_and_load / authorize_member）
│   ├── audit.rs       # 操作稽核 — 產生 AuditEntry 丟 channel（掛 with_auth / with_member_auth 內層，讀 AuthenticatedUser 或 AuthenticatedMember extension，不重複 decode；請求路徑上不碰 DB，見「操作稽核」）
│   ├── error_shape.rs # 把 layer / Router 直接吐的錯誤（408 / 413 / 405）換成統一形狀 + 落 log
│   ├── request_id.rs  # 產生（或沿用上游 `x-request-id`）追蹤 id → request extensions + task-local + 回應 header；掛最外層，span / 錯誤 body 都從這裡取
│   └── rate_limit.rs  # Redis rate limit（per IP、窗口一律 60s：tools 20、auth 登入 5 防爆破、webauthn 10、messages 5、comments 10）
├── jobs/              # 定期任務實作（每個檔案一個 pub async fn run(state: AppState)）
│   ├── aggregate_visitors.rs             # 每日 UTC 16:05（台北 00:05）落地前一台北日不重複到訪 PFCOUNT → daily_visitor_stats
│   ├── cleanup_expired_torrents.rs       # 每小時 :30 清除逾期 torrent（DB + 磁碟）
│   ├── collect_system_metrics.rs         # 每分鐘一筆系統指標 → system_metrics（保留 90 天）
│   ├── cleanup_observability.rs          # 清理逾期的 system_metrics / logs / admin_audit_logs
│   ├── cleanup_unused_images.rs          # 每小時清除 status=unused 且逾時的孤立圖片
│   ├── consume_pending_stock_change.rs   # 每分鐘消費一筆 pending stock_change
│   ├── fetch_buyback_periods.rs          # 每日 UTC 20:00 抓庫藏股計畫 HTML
│   ├── fetch_gov_tenders.rs              # 每日 UTC 23:00 依關鍵字抓政府採購網標案、新公告寄 email 通知
│   ├── fetch_historical_closing_prices.rs# 每分鐘補一筆缺起始日價格的庫藏股收盤價
│   ├── fetch_stock_day_all.rs            # 每日 UTC 20:00 抓全市場行情
│   └── sync_buyback_to_pending.rs        # 每日 UTC 20:10 將庫藏股同步為 pending stock_changes
├── repositories/      # DB + Redis 查詢函式（大檔拆成 `<name>/mod.rs` + 子模組 + `pub use`，呼叫端路徑不變）
│   ├── vocab/         # words / stats / runs / admin
│   └── stocks/        # stocks 拆成五個子模組
│       ├── changes.rs       # stock_changes table
│       ├── closing_prices.rs# stock_closing_prices table
│       ├── day_all.rs       # stock_day_all table
│       ├── buyback.rs       # stock_buyback_periods table
│       └── ex_rights.rs     # stock_ex_rights / stock_ex_rights_checked table
├── routes/            # 各領域 handler（auth, blogs, stocks, ws, images…）
│   ├── admin.rs          # /admin/* 群組（16 個 nest）：auth, users, roles, permissions, audit_logs, blogs, blog_comments, images, stocks, torrents, games, gov_tenders, stats, vocab, messages, settings
│   ├── admin_games.rs    # /admin/games — GET 即時對局總覽（需 game:read；唯讀記憶體快照，每遊戲等待/進行中桌數+在玩人數+排隊+大廳）
│   ├── admin_gov_tenders.rs # /admin/gov_tenders — GET 標案分頁列表（需 gov_tender:read；?keyword=&tender_type=&q=）、GET /types 類型清單（去重排序，供前端下拉）
│   ├── admin_stats.rs    # /admin/stats/visitors — GET 每日不重複到訪（需 stat:read；today 即時 PFCOUNT + last_n_days_unique 跨日去重 + history 歷史）
│   ├── admin_blogs.rs    # /admin/blogs — GET 分頁列表（需 blog:read；`?tag=&q=&sort=`，回 AdminBlogListItem＝**不含 markdown**；擁有者只由 session 決定、不吃 author 參數）、PUT（需 blog:update，body 只收 { markdown, tags }；tocs 由後端解析）、DELETE（需 blog:delete）；另有 tag 批次操作 PATCH /tags（改名合併，body { from, to }）與 DELETE /tags?tag=（全站移除該 tag），兩支都需 blog:update、回 { affected } 受影響文章數，且只動自己的文章（super_admin 全站）
│   └── app_settings.rs   # /admin/settings — GET（需 setting:read）、PATCH /:key（需 setting:update）
├── services/          # 業務邏輯（26 支：app_settings, audit_logs, auth, blog_comments, blogs, email, gov_tenders, images, logs, members, messages, oauth, portfolio, roles, roster, stats, stocks, system_metrics, tools, torrents, twse, users, vocab, vocab_ja, webauthn, ws；`roster` 是純函式零 IO 的排班演算法，附 13 測）
│   ├── auth.rs        # 登入/JWT + `load_identity`（顯示名/super_admin/權限的**唯一**載入點，Redis 命中＝零 DB）
│   ├── twse.rs        # TWSE API 共用層 — headers、parse、全域 semaphore(1) 防 rate limit
│   ├── stats.rs       # 到訪統計（今日 PFCOUNT + 跨日去重 + 歷史，三支併發）與 WS 採集入口
│   ├── ws/            # WS 連線層：ticket（一次性連線票）/ guard（per-IP 上限、令牌桶 MessageBudget）/ socket（handle_socket 生命週期、ping-pong、遊戲分派、斷線清理）/ connections（後台連線清單、點對點送訊）
│   ├── vocab/         # engine（純函式 + 測試）/ question / run / answer / stats / admin
│   ├── portfolio/     # crud / pricing（三層快取管線 + 上游預算）/ math（純計算 + 測試）
│   └── torrents/      # manager（TorrentManager：librqbit session + active map）/ tasks / lifecycle（watcher）/ session / download_links / storage
├── storage/           # 儲存抽象層 — 目前只有 LocalStorage
├── structs/           # serde 結構體 / request + response 型別（**API 回應型別一律住這裡，不長在 repositories/**）
│   ├── config.rs      # AppConfig、OAuthProviderConfig — 啟動時從 env 讀一次
│   ├── jobs.rs        # AppJob enum — 所有排程 job 的 variant、cron expression、run() dispatch
│   ├── pagination.rs  # PageQuery（page/per_page，上限 200）、Paginated<T>、StatusFilter（?status=，跨資源共用）
│   ├── stats.rs       # DailyVisitorStat / VisitorsQuery / VisitorsStats
│   ├── images.rs      # ImageRecord
│   ├── system_metrics.rs # SystemMetric
│   └── ws.rs          # WsEvent enum
└── utils/             # 共用工具（reqwest 包裝、date — 民國日期解析、net — client_ip、redact — query 進 log 前遮罩憑證、text — normalize_optional）
migrations/            # sqlx migration — 伺服器啟動時自動執行
```

## 狀態管理

`AppState` 包裝 `Arc<AppStateInner>`，可 clone 並注入各 handler。  
**永遠 clone `AppState`，不直接使用 `AppStateInner`。**

`AppStateInner` 欄位：
- `pg_pool` — PgPool（最多 20 連線）
- `redis_pool` — bb8 Redis 連線池
- `http_client` — reqwest Client（timeout 30 秒）
- `connections` — `Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>` 追蹤 WS 連線；`broadcast()` 直接迭代此 map 推送（無 broadcast channel）
- `audit_tx` — `mpsc::Sender<AuditEntry>` 稽核批次寫入佇列（消費端 `services::audit_logs::audit_writer`，`AppState::new()` 一併回傳 receiver）
- `storage` — `Storage` enum（目前只有 Local）
- `config` — `AppConfig`（env 設定，啟動時讀一次；只含 `jwt_secret` + OAuth `client_secret`）
- `settings` — `Arc<RwLock<HashMap<String, String>>>` runtime 設定，從 `app_settings` table 載入，可熱更新；透過 `Settings` wrapper 型別共享
- `torrents` — `TorrentManager`（librqbit Session + 進行中任務 handle map），見「Torrent 下載」
- `enabled_features` — `Arc<RwLock<Option<HashSet<Feature>>>>` instance 功能開關（None = 全開），`Settings::reload` 時由 `enabled_features` 設定 parse 出來，見「instance 功能開關」
- `games` — `GameRegistry`（每遊戲一個 hub，重啟＝全對局丟失），見「對戰遊戲框架」
- `webauthn` — `Arc<RwLock<Option<webauthn_rs::Webauthn>>>`，由 `webauthn_rp_id` / `webauthn_rp_origin` 兩個設定重建，缺漏/無效 = None
- `cpu_times` — `Arc<RwLock<Option<CpuTimes>>>` CPU 採樣基準（唯一呼叫端 `CollectSystemMetrics`），見「CPU 採樣」
（共 13 個欄位）

## 優雅關閉

`main` 收到 SIGTERM / Ctrl+C → axum 停止收新連線並等 in-flight，**同時開始計 `DRAIN_TIMEOUT`(5 秒)** → 兩者先到者勝 → sleep 700ms 等 log / audit buffer 落地 → 退出。

| 情境 | SIGTERM 後退出 | 說明 |
|------|-----------|------|
| 無進行中請求 | **0.71 秒** | 只剩 flush 那 700ms |
| 進行中的**串流回應** | **5.71 秒** + 一筆 WARN（`graceful shutdown 等了 5s 仍有連線未結束`） | `/uploads` 的 `ServeDir`、torrent 簽名下載的 `ServeFile`。hyper 的 connection future 要等 body 串完才算結束，所以沒有上限的話 graceful shutdown 等不完 → 等到 docker SIGKILL，而 **SIGKILL 那條路徑上 flush 完全不執行**，關機前的 log 與稽核直接丟（2026-08-22 實測 >20 秒不退）。2026-09-24 補上限後：下載被切斷，但 WARN 確實落進 `logs` 表 |
| 開著的 **WS 長連線** | 0.72 秒，本來就不卡 | ⚠️ 反直覺但已驗證：hyper 的 `UpgradeableConnection` 在 upgrade 當下 `inner` 就變 `None`、future 隨即 Ready，所以 axum 那條 per-connection task 早就結束了，WS **從來不算 in-flight**。<br>⇒ **不要為了「讓 WS 主動收線」去改 `services/ws/socket.rs` 或往 `AppState` 塞關機訊號**：那條路 2026-08-22 走過、寫完才發現前提是錯的 |

- 實作：`main.rs` 用 `watch` channel 把「收到訊號」同時交給 `with_graceful_shutdown` 與 `drain_deadline`，`select!` 兩者。
- ⚠️ **`DRAIN_TIMEOUT` + 700ms 必須明顯小於 docker 的 stop grace period**（backend 吃預設 10 秒，`deploy/docker-compose.yml` 沒另設）。調到接近 10 秒，行程仍會被 SIGKILL 收掉，flush 照樣跑不到。要加長排水時間就得同時在 compose 設 `stop_grace_period`。

重現方式（本機需有 PG + valkey，見「本機端到端驗證」）：

```bash
head -c 20000000 /dev/urandom > uploads/_t.bin
cargo run &
curl -s --limit-rate 100k http://127.0.0.1:<port>/uploads/_t.bin -o /dev/null &
sleep 2; kill -TERM <pid>   # 預期約 5.7 秒退出，log 有 WARN 與 "server stopped, flushing logs"
```

## Runtime 設定（app_settings）

`app_settings` table（key/value/description/category）儲存可熱更新的設定，不需重啟伺服器。

`GET /admin/settings` 回傳 `BTreeMap<String, Vec<AppSetting>>`，以 `category` 為 key 分組（順序穩定）。

| key | category | 說明 | 生效時機 |
|-----|----------|------|----------|
| `site_theme` | `appearance` | 網站風格主題（forest / ocean / sky / sunset / sakura / grape / mono / `auto`），PATCH 驗證值、非法回 422；`auto` = 走每日輪播 | 即時 |
| `default_color_mode` | `appearance` | 全站深淺色預設（light / dark / system），PATCH 驗證值、非法回 422 | 即時 |
| `theme_rotation` | `appearance` | 每日輪播主題對應表，JSON 字串 `{"0".."6": <theme>}`（0=週日..6=週六），site_theme=`auto` 時前端依當天星期查表；PATCH 驗證 key 齊全＋值合法（拒 `auto`）非法回 422 | 即時 |
| `home_features` | `homepage` | 首頁功能卡片（JSON 字串陣列＝顯示與排序，空陣列＝全部隱藏）；PATCH 只驗形狀（字串陣列、不重複、≤50 項），**不驗 key 名**——功能清單由前端 `libs/home-features.ts` registry 定義，未知 key 前端忽略，新增卡片只改前端。⚠️ 與隔壁 `enabled_features` 剛好相反（那個嚴格驗 key 名，權威在後端 `Feature` enum）：純展示的清單容忍前端當權威，會影響 API 行為的開關不行 | 即時 |
| `enabled_features` | `features` | instance 功能開關（`all` = 全開，或 JSON 字串陣列白名單；見「instance 功能開關」一節） | 即時 |
| `webauthn_rp_id` | `platform` | Passkey RP ID（裸網域；有 passkey 後不可改），平台保留 key。與 `webauthn_rp_origin` 有跨欄位不變式（見批次 PATCH），整組換網域要走 `PATCH /admin/settings` 一次寫兩個 | 即時 |
| `webauthn_rp_origin` | `platform` | Passkey origin（登入頁所在前端來源 URL），平台保留 key | 即時 |
| `new_user_default_roles` | `user` | 新建管理員預設勾選的角色（逗號分隔角色名稱，migration 預設 `admin`），平台保留 key —— 能改它等於能決定新管理員的權限 | 即時 |
| `gov_tender_keywords` | `gov_tender` | 政府採購網標案追蹤關鍵字（逗號分隔，留空 = FetchGovTenders 跳過） | 即時 |
| `google_client_id` | `oauth` | Google OAuth Client ID | 即時 |
| `google_redirect_url` | `oauth` | Google OAuth Redirect URL | 即時 |
| `github_client_id` | `oauth` | GitHub OAuth Client ID | 即時 |
| `github_redirect_url` | `oauth` | GitHub OAuth Redirect URL | 即時 |
| `line_client_id` | `oauth` | LINE OAuth Client ID | 即時 |
| `line_redirect_url` | `oauth` | LINE OAuth Redirect URL | 即時 |
| `cors_allowed_origins` | `cors` | CORS 允許來源（逗號分隔）。PATCH 會驗每項為 `http(s)://host[:port]` 且**拒絕 `*`** —— `AllowOrigin::list` 遇到 `*` 會 panic，而 CORS 只在啟動時讀，設錯要到下次重啟才炸成無限重啟迴圈 | 重啟 |
| `upload_base_url` | `storage` | Upload Base URL | 即時 |
| `image_webp_quality` | `storage` | 後端 WebP 重編碼品質（1–100，只後端讀，不公開） | 即時 |
| `image_client_compress` | `storage` | 前端上傳前壓縮開關（`true`/`false`，走 PUBLIC_KEYS 下發） | 即時 |
| `image_client_quality` | `storage` | 前端壓縮品質（1–100，公開） | 即時 |
| `image_client_max_edge` | `storage` | 前端壓縮長邊上限 px（公開） | 即時 |
| `smtp_username` | `notification` | Gmail 寄件帳號 | 即時 |
| `smtp_password` | `notification` | Gmail App Password。**write-only**：`GET /admin/settings` 回 `********`，PATCH 仍可寫入（`SECRET_KEYS`） | 即時 |
| `notify_email` | `notification` | 通知收件信箱（空白 = 同寄件帳號） | 即時 |
| `torrent_max_active` | `torrent` | 同時下載的 torrent 數上限（預設 2） | 即時 |
| `torrent_retention_days` | `torrent` | completed/failed 保留天數，逾期自動清（預設 7） | 即時 |
| `torrent_max_total_size_gb` | `torrent` | torrent 目錄容量上限，超過拒收新任務回 507 | 即時 |
| `torrent_link_ttl_minutes` | `torrent` | 下載連結效期（分鐘，預設 180） | 即時 |
| `torrent_metadata_timeout_seconds` | `torrent` | magnet metadata 解析的**讓位檢查間隔**秒數（預設 180，clamp 30–3600）：到點只在有任務排隊時才放棄本輪 | 即時（下次啟動嘗試） |
| `log_db_level` | `observability` | `logs` 表落地門檻（`ERROR` / `WARN` / `INFO`，預設 WARN；PATCH 驗值、非法回 422）。**只能在 `RUST_LOG` 這個天花板底下調**，見「應用日誌」 | 即時 |

讀取：`state.get_settings().get("key")`（sync，`std::sync::RwLock`）  
更新：`PATCH /admin/settings/{key}` body `{ value }`，service 層更新 DB 後自動呼叫 `settings.reload(pool)`

**批次更新**：`PATCH /admin/settings` body `{ values: { key: value, … } }`，全部驗證通過（含跨欄位）才在同一 transaction 寫入、最後 reload 一次。存在的理由是**互相約束的設定組**：逐 key PATCH 的中間狀態必然違反不變式，沒有這支就只能把檢查搬到前端（那等於沒有檢查）。目前唯一的跨欄位規則是 `validate_webauthn_pair`（rp_id 必須是 rp_origin hostname 本身或其上層網域；任一邊為空＝尚未設定完成，不檢查）。權限逐 key 判斷，帶到保留 key 就需要 `platform:update`。

`GET /settings/public`（無認證）回傳白名單設定（`services/app_settings.rs` 的 `PUBLIC_KEYS`），直接讀記憶體 map 不打 DB。

**值在出站前轉成該有的型別**（2026-08-07）：`PUBLIC_KEYS` 是 `(key, PublicKind)` 表，`Text` 原樣、`Bool` → JSON 布林、`Int` → JSON 數字、`Json` → parse 成物件/陣列。`app_settings.value` 是 text 那是**儲存層**限制，不該滲進 API 契約——原本整包回字串，於是 `theme_rotation` / `home_features` / `enabled_features` 是「JSON 字串包在 JSON 裡」，每個前端消費端都要再 parse 一次並各自處理壞值。**轉不動的值原樣退回字串**（`enabled_features` 的合法值 `all` 就不是 JSON，靠這條走），所以前端 resolver 仍要吃得下字串形。新增公開設定：在 `PUBLIC_KEYS` 加一行（key + 出站型別）。

新增設定：在 migration 插入新行（含 `category`）即可，不需改 Rust 程式碼。

## instance 功能開關（enabled_features）

每套部署可開關的功能（instance-per-merchant 產品化基礎），設計定案見 `docs/2026-07-19-enabled-features-plan.md`。

- **key 權威 = `structs/features.rs` 的 `Feature` enum**（10 個：blog / tools / roster / games / stocks / portfolio / vocab / torrents / gov_tenders / message）。核心永不受控：admin 基礎、members/oauth、ws 基礎連線、logs/metrics、uploads。
- 設定值：`all` = 全開（本站預設，未來新功能自動開）；JSON 字串陣列 = 白名單（商家 instance，新功能預設關）。PATCH 嚴格驗證（未知 key / 重複 / `portfolio` 未帶 `stocks` 皆 422）。
- **平台保留 key**（`services/app_settings.rs` 的 `RESERVED_KEYS`，目前 **4 個**：`enabled_features`、`webauthn_rp_id`、`webauthn_rp_origin`、`new_user_default_roles`——最後一個決定「新建管理員預設掛哪些角色」，那是平台層的權限決策，不該只要 `setting:update` 就能改，否則等於另開一條指派角色的門）：GET `/admin/settings` 無 `platform:read` 者直接濾掉、PATCH 需 `platform:update`（一般 key 仍走 `setting:update`）。商家管理員拿 `setting:*` 管日常設定，碰不到保留 key；前端專頁 `/admin/platform`。新增保留 key = `RESERVED_KEYS` 加一項。
- 檢查是 sync set lookup：`Settings::reload()` 時 parse 成 `Option<HashSet<Feature>>`（None = 全開），`state.get_settings().feature_enabled(Feature::X)`。熱更新即時生效、不重啟。
- **三個攔截點**：
  1. routes — `routes.rs` 的 `with_feature(state, Feature::X, router)`（與 `with_auth` 對稱），未啟用回 **404**（包在 auth 外層，連 401 都不回、不暴露功能存在）
  2. jobs — `AppJob::feature()`（core 回 None），`scheduler.rs` 每次觸發時檢查
  3. ws 遊戲 — `dispatch_game` 入口檢查 `Feature::Games`，關閉回 `error` 信封 `{reason: "feature_disabled"}`；watcher 照常 spawn（掃空 map 零成本，換熱開關不重啟）。啟動期唯一 gate：torrents 關閉時跳過 `sync_active`
- **新增可開關功能**：`Feature` 加 variant（+ `as_str`/`from_key`/`ALL`）→ route 掛 `with_feature` → 有 job 就補 `AppJob::feature()` → 前端 `libs/enabled-features.ts` 的 `BACKEND_FEATURES` 同步加 key。

## stock_changes 資料流

`stock_changes` table 的 `status` 值：`pending` → `completed` / `failed`  
`start_date` / `end_date` 欄位型別為 `DATE`（西元）。

### 寫入 pending 的入口

| 入口 | 說明 |
|------|------|
| `SyncBuybackToPending` | 每日 UTC 20:10 自動從 `stock_buyback_periods` 同步；`pending` 狀態的記錄若 `end_date` 有異動會自動更新，`completed` / `failed` 不動 |

### 消費 pending

`ConsumePendingStockChange`（每分鐘）：
1. 取一筆 `status = pending` 且 `end_date <= 今日` 的資料
2. 從 `stock_closing_prices`（DB cache）取起始/結束收盤價；cache miss 才打 TWSE
3. 股票名稱從 `stock_day_all` 取，不額外打 TWSE
4. upsert 結果，status 改為 `completed`；失敗則改為 `failed`，廣播 `WsEvent::StockFailed`
5. 成功廣播 `WsEvent::StockCompleted`

### 其他操作

| Endpoint | 說明 |
|----------|------|
| `PATCH /admin/stocks/changes/{id}/pending` | 將指定 id 重設為 `pending`（清空價格欄位），回 204 |

### /admin/stocks 路由一覽

| Endpoint | 說明 |
|----------|------|
| `GET /changes` | stock_changes 分頁列表（`?status=&page=&per_page=`） |
| `PATCH /changes/{id}/pending` | 重設為 pending |
| `GET /day_all` | 全市場行情（分頁） |
| `GET /buyback_price_gaps` | 未完成庫藏股價差 |
| `GET /buyback_periods` | 庫藏股計畫列表 |

## Torrent 下載（/admin/torrents）

磁力連結 → librqbit 下載到 `TORRENT_PATH/<info_hash>/` → 簽名 URL 走 HTTPS 取檔。
狀態流：`pending` → `downloading` → `completed` / `failed`。細節見 `docs/old/torrent-downloader-workflow.html`，前端規格見 `docs/old/torrent-frontend-spec.md`。

| Endpoint | 權限 | 說明 |
|----------|------|------|
| `POST /` | `torrent:create` | 收 magnet，回 201；重複 409、格式錯 422、容量滿 507 |
| `GET /?status=&page=&per_page=` | `torrent:read` | 分頁列表 |
| `GET /storage` | `torrent:read` | 磁碟剩餘空間（statvfs）+ torrent 配額用量 |
| `GET /{id}` | `torrent:read` | 詳情；進行中任務附 `live` 即時進度 |
| `POST /{id}/download_links` | `torrent:read` | 產生短效簽名下載連結（效期 `torrent_link_ttl_minutes`） |
| `GET /{id}/files/{file_index}?token=` | 簽名 token | 串流下載，支援 Range；**不掛 JWT middleware**，但 token 內嵌發行者 email，下載時即時重查 `torrent:read` 權限 |
| `PATCH /{id}/pending` | `torrent:create` | failed/completed 重設重跑 |
| `DELETE /{id}` | `torrent:delete` | 停任務 + 刪 DB + 刪磁碟 |

要點：
- 併發上限 `torrent_max_active`，超過排隊（pending），完成/失敗/刪除自動補位（`sync_active`），另有每小時 cleanup job 順帶補一次位當保險
- ⚠ **magnet 的 metadata 解析發生在 `session.add_torrent()` 內部且 librqbit 沒有逾時**——沒 peers 就無限期卡住。所以 metadata 的逾時（`DEFAULT_METADATA_TIMEOUT_SECONDS`，可由 `torrent_metadata_timeout_seconds` 蓋掉）必須包在 `add_torrent` 這一層（包在後面的 `wait_until_initialized` 是抓不到的，那時 metadata 早就到手了）；`INIT_TIMEOUT` 才是後者，等的是既有檔案的 piece 驗證
- **併發名額在 `add_torrent` 之前就佔位**（`active` map 存 `Slot { task, handle: Option }`，`handle` 要解析完才有）。不先佔位的話解析期間那筆任務不算在併發內，會被重複啟動、名額也算不準；`sync_active` 對每筆 spawn 獨立 task（不可序列 await，第一筆卡住會拖死整排）。刪除時 `slot.task.abort()` 才砍得掉還在解析中的任務
- **讓位只在有人排隊時發生**（`queue_pressure`：`count_resumable() > torrent_max_active`）。`torrent_metadata_timeout_seconds` 不是硬逾時而是**檢查間隔**：`start_torrent` 對同一個 pinned `add_torrent` future 反覆 `timeout`，到點沒人排隊就 `continue` 繼續等（future 沒被 drop，DHT 查詢與半握手的 peer 全留著），只有真的有任務排不進名額才放棄本輪。**不要改成硬逾時或短間隔輪替**：magnet metadata 解析的進度全在 `add_torrent` 那個 future 裡（librqbit 內部，沒有可觀測的中間狀態），abort 等於從零重來，冷門種子需要的是不被打斷的時間
- **輪替帳本**：`torrents.last_attempt_at` / `attempt_count`（`run_torrent` 開頭 `mark_attempt` 記一次），`list_resumable` 排序 = 續傳的 downloading 優先 → 沒試過的 → 最久沒試的。讓位且 `attempt_count < MAX_METADATA_ATTEMPTS`(3) 就留在 `pending` 只更新 `error`（廣播 `torrent_retrying`，前端刷列表、pending 帶 error 顯示灰字提示），額度用完才 `set_failed`。拿到 metadata（`set_downloading_metadata`）與手動 `reset_pending` 都會把 `attempt_count` 歸零 —— 所以「沒人排隊時一直等」不會累積次數、不會被判失敗
- 下載完成即從 session 移除（停止做種），檔案保留
- 重啟 resume：啟動時 `routes.rs` spawn `sync_active`，downloading 任務以 `overwrite: true` 重加（驗證既有 piece 續抓）
- 進度不落 DB — watcher 每 5 秒讀 stats，變動才廣播 `WsEvent::TorrentProgress`
- 下載完成寄 email 通知（沿用 `notification` 類 SMTP 設定，未設定自動跳過）
- 下載 token 是獨立 JWT claims（purpose=`torrent_download`），與 admin/member token 不互通

## API 慣例

- 分頁一律 `?page=1&per_page=50`（`structs/pagination.rs` 的 `PageQuery`，`per_page` 上限 200），handler 內 `page.to_limit_offset(預設值)` 轉 SQL limit/offset
- POST 建立資源回 `201`（有 entity 就帶 body），更新/刪除無內容回 `204`
- 路由命名 RESTful：資源名詞 + HTTP method，不用 `get_*` / `fetch_*` 動詞前綴。**handler 函式名同理**（2026-07-31 已把 24 個 `get_*` 收斂完）：單一資源檔用裸動詞（`list` / `detail` / `create` / `update` / `delete`，見 `routes/portfolio.rs`），需要區分時才加名詞後綴（`list_words` / `delete_message`，見 `routes/admin_vocab.rs`）。**選新名時照該檔既有的命名家族走，不要引入第三種風格**
- `/tools/*` 是**例外**：`convert_text` 本質是計算工具而非資源，路徑刻意保留動詞，硬套名詞路徑語意更差（同群的 `new_password` 已於 2026-08-30 移除 —— 密碼改由瀏覽器端 `crypto.getRandomValues` 產生，見 `frontend/libs/password.ts`）
- **錯誤回應只有一種形狀**：`errors.rs` 的 `{ code, message, details?, request_id }`。`fallback`（未知路徑）與 `with_feature`（功能關閉）都回 `AppError::from(RequestError::NotFound)`，不要再寫 `(StatusCode::X, "字串")` 或 `(StatusCode::X, Json(json!(...)))` 這種裸回應——會讓客戶端要 parse 兩種格式，也拿不到 `request_id`。限流的 429 走 `RequestError::TooManyRequests`（2026-07-31 補的 variant；在那之前 `rate_limit` middleware 自組 JSON）。**要回新的狀態碼就先去 `errors.rs` 加 variant**，不要在 middleware 或 handler 裡自己組 body

### 「一種形狀」是怎麼守住的（2026-08-09 補完最後兩個破口）

在此之前，**沒有經過 handler 的錯誤全部漏在外面**：少填一個 JSON 欄位、路徑參數型別不符、body 超過 10MB、method 對不上 —— 這幾種回的是 axum 的純文字，body 裡沒有 `request_id`、也不落 log，而它們正是使用者回報量最大的那類。現在有三道：

1. **`extract.rs` 的 `Json` / `Query` / `Path`** —— 全站一律用這裡的版本，rejection 轉成 `AppError`。⚠️ **`clippy.toml` 把 `axum::Json` / `axum::extract::Query` / `axum::extract::Path` 列為禁用型別**，CI 跑 `cargo clippy --all-targets -- -D warnings`，所以用錯是紅燈不是提示 —— 漏用沒有任何執行期徵兆，只能靠 lint 擋（導入時就靠它抓到 `routes/tools.rs` 一處漏網）。例外只有 `extract.rs`（包裝層本身）與 `errors.rs`（組錯誤 body），兩處以模組層級 `#![allow(clippy::disallowed_types)]` 標註。
2. **`middleware/error_shape.rs`** —— 收 layer / Router 直接吐的那些（`TimeoutLayer` 的 **408**、`RequestBodyLimitLayer` 的 **413**、Router method 不符的 **405**）。判斷條件刻意是「4xx/5xx 且 content-type 不是 JSON」而**不是列舉狀態碼**：列舉的話多一個 layer 就又靜默漏一種。原回應的 header 會保留（405 的 `Allow` 是協定要求的）。
3. **`errors.rs::handle_panic`** —— handler panic（見「應用日誌」）。

**狀態碼一律沿用原本那個**（`RequestError::Rejection { status, message }` 帶著 status 走），只換 body。逐一映射到既有 variant 會在上游新增拒絕原因時悄悄改掉狀態碼。實測：422（缺欄位）/ 400（JSON 語法、Path、Query）/ 415（缺 content-type）/ 413 / 405 全部維持原狀態碼，body 換成統一形狀且帶 `request_id`，並落一筆 WARN。⚠️ **只有 path 參數的 rejection 例外**：它走獨立的 `RequestError::PathRejection`（body / 狀態碼與 `Rejection` 完全相同，分開只為 log 分級）落 **debug** 不落 WARN —— `/blogs/null` 這種來源是壞連結與掃站，見「分級判準」。
### 請求逾時：60 秒（2026-08-09）

`routes.rs` 掛 `tower_http::timeout::TimeoutLayer`，逾時回 **408**（body 由 `error_shape` 正規化，
帶 `request_id`、落一筆 WARN）。沒有它時，一個卡住的 handler（上游不回、DB 慢查詢、鎖等待）
會一直佔著 tokio worker 與那條 PG 連線 —— 1 核 1G 上幾個就吃光。

- **60 秒是對齊 nginx，不是隨手挑的**：`deploy/nginx/conf.d/api.kawa.homes.conf` 的 `location /`
  吃預設 `proxy_read_timeout 60s`，超過的請求 client 早就收到 504，backend 再跑下去純屬浪費。
- ⚠️ **不要調更短**：api vhost 開了 `proxy_request_buffering off`，圖片上傳的 client body 是
  邊傳邊進 handler 的，**計時包含使用者的上傳時間** —— 10MB 走慢速行動網路可以超過半分鐘，
  短逾時會把正常上傳打成 408。
- 對長連線無影響：計時只到「回應產生」為止。WS 的 101 與 torrent 下載的 `ServeFile`
  都是先回 header 再串流，串流時間不算在內。
- 掛在 `error_shape` **內層**（程式碼順序在其前）才吃得到正規化；在 `TraceLayer` 內層才有 span。

- `GET /health` 是存活探針（公開、回 `200 {"status":"ok"}`），**刻意不查 DB / Redis**——探針只回答「行程活著且在收請求」，加了外部依賴會讓 DB 抽風時整站被監控判死、也給每次探測都上一份 DB 負擔。根路徑 `/` 無路由（回 404 是正常的），監控要打 `/health`

## WebSocket 推送

`AppState::broadcast(event, data)` 廣播事件給所有連線中的 WS 客戶端。
`AppState::broadcast_to_admins(event, data)` 只推給已通過 admin 驗證的連線 —— **含個資（IP / email）的事件一律走這個**（目前 `user_joined` / `user_left`）。

```rust
state.broadcast(WsEvent::StockCompleted, serde_json::json!({ "stock_no": "2330" }));
```

事件型別定義於 `src/structs/ws.rs`（`WsEvent` enum）。新增事件：加 variant + `as_str` match arm，**並同步前端 `types/ws.ts` 的 `WsEventType`**（兩邊一一對應）。**事件名不要在 route 裡手寫字串**——`admin_message` 原本就是那樣繞過 enum，於是前端對照表也跟著漏掉它與 4 個 torrent 事件（2026-08-07 收齊）。

### 送出順序：同一連線的多則訊息必須走 `send_many` / `send_outbox`

`state.rs` 的送出 API 有三個，差別**只在順序保證**：

| API | 用途 | 順序 |
|-----|------|------|
| `send_to(addr, msg)` | 單則點對點 | 不適用 |
| `send_many(addr, msgs)` | 同一連線多則 | **保證按 `msgs` 順序抵達** |
| `send_outbox(outbox)` | 一整批（遊戲框架的 `flush` 走這個） | 同收件人保證順序，不同收件人平行 |

⚠️ **不要為了「簡單」把一批訊息拆成逐則 `send_to`**（2026-08-06 修掉的就是這個）：每次 spawn 出去的 task 各自搶該連線的 sender lock，取得順序不保證等於 spawn 順序。壞掉的是依賴順序的協定 —— 阿瓦隆開局同一批裡有私有 `role_assigned` 與公開階段訊息，而**前端的 `your_seat` 只從前者取**，倒序抵達就會在還不知道自己是誰的狀態下處理階段更新；2 人局的 `move_made` 與 `game_over` 同理。`send_many` 的做法是整批只 spawn 一個 task、sender lock 一次取得送完才放（順帶擋掉別人插進批次中間），spawn 數也從 O(訊息) 降成 O(連線)。

分組邏輯抽成純函式 `state.rs::group_by_addr`（附 3 個順序測試）—— 那是這件事唯一可單測的部分。

### Ping / Pong：半開連線靠 Pong 逾時收掉

後端每 `PING_INTERVAL_SECONDS`(30) 發一次 Ping，`recv` 迴圈記下最後收到 Pong 的時間；連續 `PONG_TIMEOUT_SECONDS`(75) 沒有 Pong 就結束 ping task → `select!` 收到 → abort recv → `cleanup_connection`（含各遊戲斷線處理）。實測：完成握手後完全不回應的 client 在 **90 秒**被判死並清掉（第三次 tick）。

**連線結束留一行 INFO 摘要**（2026-09-07，`handle_socket` 尾端）：`reason` / `duration_secs` / `messages`（只算 Text/Binary，控制帧不計），識別欄位在 span 上。`reason` 的值：`client_close`（收到 Close 帧）/ `stream_end`（TCP 直接斷、連 Close 都沒送）/ `recv_error` / `flood` / `pong_timeout` / `ping_failed` / `recv_task_error`｜`ping_task_error`（task panic 或被 abort）。

- **為什麼是 INFO 不是 debug**：生產的 `RUST_LOG` 天花板是 `info`，WS 這邊原本清一色 `debug!`，於是在生產**根本不存在**（EnvFilter 就擋掉，stdout 與 `logs` 表兩邊都沒有）——「一群人同時掉線」事後完全無跡可循。量級是每條連線一行；要落 `logs` 表仍需把 `log_db_level` 調到 INFO，但 stdout 一定有。
- **逐則收訊/送出失敗維持 debug 不變**（理由見上面的分級判準，那是關分頁的常態）。摘要不是取代它們，是補上「這條連線整體怎麼收場」。
- **`handle_socket` 掛在一條 `ws` span 上**（`conn` / `ip` / `email` / `request_id`，`instrument` 顯式帶過去 —— `on_upgrade` 的 future 由 hyper 在另一個 task 驅動，span context 不會自己跟）。在此之前 upgrade 之後的所有 WS log 既沒 request_id（task-local 留在握手那個 task）也沒結構化欄位，`logs` 表裡只剩一段夾著 SocketAddr 的字串，對不回任何連線或握手請求。

**為什麼不能只靠 send 失敗**：對端消失但 TCP 沒斷（拔網路、手機睡眠、NAT 逾時）時寫入會先進 kernel buffer 而「成功」，可能好幾分鐘才回報錯誤。期間那條連線會一直掛在 `connections` map（後台連線列表看得到）與遊戲桌位上（對手在等一個永遠不會來的走步）。瀏覽器的 WS 實作會自動回 Pong，所以收不到就是真的沒人在了。

### 連線防護：per-IP 連線上限 + 每連線收訊令牌桶（2026-08-24）

`/ws` 是**匿名公開**端點，而 `middleware/rate_limit.rs` 只保護 HTTP —— 在此之前握手前後都零額度檢查。1 核 1G 上這是兩個實際可用的資源耗盡面：① 一個腳本開 N 條連線，每條都能在遊戲 hub 佔一張桌／一間房 ② 單條連線用 while 迴圈灌訊息，每則都去搶 hub 的 mutex 並向整個大廳訂閱集廣播。兩道的判斷都在 `services/ws/guard.rs`（純記憶體，握手路徑不多一次 Redis IO；令牌桶 `MessageBudget` 與 per-IP 上限常數住這裡，**丟棄計數與收線門檻 `MAX_DROPPED_MESSAGES` 在 `services/ws/socket.rs`** —— 那是 socket task 的迴圈）：

| 防線 | 值 | 行為 |
|------|----|------|
| per-IP 連線數 | `MAX_CONNECTIONS_PER_IP` = 12 | **握手前**擋掉，回 429，不進 `connections`、不記到訪、不佔桌。**admin（ticket 身分）不受限**（後台會開多分頁，且該路徑已要求 `ws:read`） |
| 每連線收訊 | `MessageBudget`：容量 30、每秒回補 6 | 超量的 Text/Binary **丟掉不解析**（控制帧不計費），第一則丟棄時回一則 `{type:"error",data:{reason:"rate_limited"}}` 並記 WARN；累計丟到 `MAX_DROPPED_MESSAGES`(200) 才收線 |

- **超量不立刻收線是刻意的**：前端 `ws-context` 會自動重連，一超量就關等於送對方一個重連迴圈。丟棄已經讓灌訊息毫無收益（丟在 JSON parse 之前）。
- 令牌桶是 socket task 的區域變數 —— 無共享狀態、無鎖、無 Redis。附 3 個測試（容量內全過 / 逾時回補 / 閒置再久也只補到上限），`refill_at` 可注入時間點。
- `rate_limited` 走 `envelope` 而**不是** `game_envelope`（那是連線層級的事，不屬於任何遊戲）。前端 `_shared/useRoomBase` 因此對 `error` 放行「不帶 game」的訊息，否則會被 game 分流濾掉。
- 遊戲 hub 另有數量上限：`common/service.rs` 的 `MAX_TABLES`(200，`create_table` 與 `join_queue` 都檢查) 與 `common/room.rs` 的 `MAX_ROOMS`(100)，超過回 `too_many_tables` / `too_many_rooms`。有了 per-IP 上限後這兩個從單一 IP 打不到，是給多來源的縱深。
- 2026-08-24 實測（本機拋棄式環境，Node 22 內建 WebSocket 當 client）：第 13 條連線握手被拒並留下 `WS 連線數超限：ip=… 已有 12 條` 的 WARN；連送 80 則 `list_tables` 收到 `rate_limited` 且連線仍 OPEN；關掉連線後額度立即回收。

### WS 連線的 admin 身分：一次性 ticket

JWT **不走** WS URL query（會進 access log）。流程：登入中的 admin 打 `POST /ws/ticket`（`authorize_and_load` + **`ws:read` 權限**保護）→ 後端發 UUID ticket 存 Redis `ws:ticket:{ticket}`（value = admin **顯示名 `users.name`**，30 秒 TTL —— admin 登入識別是 name 不是 email，見「身份識別」）→ client 以 `/ws?ticket=` 握手 → 後端 `GETDEL` 一次性消費取回那個 name。⚠️ 程式碼裡這個值沿用 `user_email` 這個舊名（`connections` map、`broadcast_to_admins` payload、`admin_audit_logs.user_email` 都是），**欄位名是 email、內容是 name**。無 ticket / 票失效 = 匿名連線（前台訪客即此路徑）。

⚠️ **`/ws/ticket` 的權限門檻必須與 `GET /ws/connections` 一致（同為 `ws:read`）**：ticket 換來的連線會被標成 admin 身分，因此收得到 `broadcast_to_admins` 推的 `user_joined` / `user_left` —— 那兩則的 payload 含 `real_ip` / `user_email` / `user_agent`。少了這道檢查，沒有 `ws:read` 的商家管理員HTTP 端查不到連線清單，卻能改走 WS 即時收到每個訪客的個資（2026-07-31 修）。admin 端點是 `GET /ws/connections`、`POST /ws/messages`、`POST /ws/ticket`；前兩者原本叫 `get_online_connections` / `say_something_to_someone`，同日改成資源路徑（前端同一 commit 已同步）。

前端收到格式：`{ "type": "stock_completed", "data": { ... } }`

### 統一信封

所有送出端走 `structs/ws.rs` 的 `envelope(type, data)`（通知）／`game_envelope(game, type, data)`（遊戲）序列化，杜絕格式長歪。應用層訊息一律 `{ type, data, game? }`（`game` 僅遊戲訊息帶）。`process_message` 收到非 JSON / 未知訊息**一律忽略**（無 echo）。admin 點對點直送 = `{ type: "admin_message", data: { content, from } }`（`from` = 管理員 email，型別走 `WsEvent::AdminMessage`）。

## 對戰遊戲框架（複用 `/ws`，點對點）

Server-authoritative 對戰，匿名可玩。**七遊戲**：回合制 2 人 — **象棋 chess / 五子棋 gomoku / 暗棋 banqi / 西洋棋 western_chess / 圍棋 go**；N 人獨立子系統 — **阿瓦隆 avalon**（社交推理）、**農場經營 farm**（worker-placement，完全資訊）。**WS 協定以 monorepo 根 `protocol/games-wire.md` 為準**（進版控；2 人通用流程 + 共通信封 + 各遊戲差異 + 阿瓦隆 / 農場完整協定，2026-09-25 由原本不進版控的 `docs/old/` 四份整合而來）。改協定＝同一個 commit 改前後端與該檔。

- **共用框架在 `src/games/common/`**：`GameEngine` trait（`initial`/`turn`/`side_label`/`try_move`/`status`/`hints`）+ 泛型 `HubInner<E>`/`service::*`。大廳/桌位/配對/Fischer 計時/斷線/timeout_watcher **全泛型**，新遊戲只寫純引擎 + adapter（`impl GameEngine`），零 glue。
- 各遊戲：純引擎 `games/<game>/engine.rs`（零 WS）+ **測試在 `games/<game>/engine/tests.rs` 子模組**（每個遊戲都有，不在 engine.rs 內）+ adapter `games/<game>/game.rs`。chess 14 測 / gomoku 6 測 / banqi 12 測 / western_chess 10 測 / go 9 測 / avalon 13 測 / farm 12 測，合計 **76**（2026-08-17 核對相符）。
- **註冊表收斂**：`games/registry.rs` 的 `AnyHub` enum（每遊戲一 variant）+ `GameRegistry`（`HashMap<&str, AnyHub>`）。`AppState.games` 只存一個 `GameRegistry`，重啟＝全對局丟失。enum match 各臂具體型別→單型化呼叫泛型 `service::*`，**無 dyn / 無 async-trait**。
- 每遊戲 hub = `GameHub<E>`（`Arc<Mutex<HubInner<E>>>`：queue / tables / conn_table / lobby）。`Table` 兩態 `Waiting{host}` / `Playing(Game)`；一條連線同時只能在 {queue, table} 其一。
- 兩種進場：**大廳自選桌**（建桌 / 挑等待桌）＋ **快速隨機配對**（單一佇列），並存。多局並發，不限桌數。
- **不經 `WsEvent` enum / `broadcast()`**：用 `AppState::send_to(addr, msg)` 點對點送；大廳更新只送 `lobby` 訂閱集。整批 outbox 一律走 `flush` → `AppState::send_outbox`（**保證同一收件人的訊息按 push 順序抵達**，見下方「送出順序」）。
- **信封 `{ game, type, data }`，`game` 欄必填**。`ws.rs::dispatch_game` 用 `state.games().get(game)` 取 `AnyHub` 後 `hub.handle()`；斷線 `for hub in games().all() { hub.disconnect() }`；watcher 同樣迴圈 spawn（routes.rs）。state/ws/routes 全 game-agnostic。
- 計時：標準 Fischer，初始 300000ms +30000ms/步（`GameEngine` const 可覆寫）。
- **合法步提示 `hints`（2026-08-24）**：`GameEngine::hints(&self) -> Option<Value>`（預設 `None`），`service` 在 `match_found` 之後與每次 `move_made` 之後**只推給當前輪到的那一方**（同批送出，順序有保證；對局已結束不推）。**這不是權威** —— 判定仍只有 `try_move`；但提示必須由 server 出，否則前端得複刻棋規（規則兩份必然漂移）。形狀各遊戲自定：象棋／西洋棋／暗棋 = `{ moves: { "col,row": [[col,row]…] } }`（暗棋另有 `flips`）、圍棋 = `{ forbidden: [[col,row]…] }`（**只送禁著點**：可下點幾乎是整盤空點，19×19 每手兩份 3KB，而「空不空」前端自己看得出來，需要規則判斷的只有自殺與劫）、五子棋**不實作**（合法點 = 所有空點）。實作來源：象棋 `engine::legal_moves(state, sq)`、西洋棋 `engine::legal_moves(state)`（同一 (from,to) 的多個升變目標併成一格）、圍棋 `engine::is_legal` 掃空點、暗棋**在複本上試走**（引擎只有 `apply_action` 這個驗證＋套用的入口，4×8 盤面一輪最多 512 次試走，成本可忽略；規則仍只有引擎那一份）。2026-08-24 實測：西洋棋開局回 10 子 / 20 步（與棋理相符）、暗棋開局 32 flips / 0 moves（未定色）、圍棋空盤 `forbidden: []`。
- 共用 type：上行 `join_lobby`/`list_tables`/`create_table`/`join_table`/`leave_table`/`join_queue`/`leave_queue`/`move`/`resign`；下行 `table_list`/`lobby_update`/`table_created`/`queued`/`match_found`/`move_made`/`hints`/`game_over`/`error`（象棋／西洋棋另有 `check`）。move data 與 color 標籤/reason 各遊戲不同，見 wire 協定。
- **新增遊戲（回合制 2 人）**：① 寫 `games/<game>/engine.rs`（純規則 + 測）② `game.rs` `impl GameEngine`（`hints` 選做，有規則限制的遊戲建議做） ③ `games/mod.rs` 加 `pub mod` ④ **只動 `games/registry.rs`**：`AnyHub` 加 variant、**四個 method**（`handle` / `disconnect` / `snapshot` / `spawn_watcher`）各加一臂、`new()` 註冊一行 ⑤ 更新 wire 協定。state/ws/routes **不必改**。
- **非 GameEngine 子系統（avalon / farm）**：N 人、不走 2 人 hub/service，改走 `common/room.rs` 的泛型 N 人房框架 —— `RoomKind` trait（marker type 提供 NAME/人數上下限/預設房名/選項解析/room_snapshot 附加欄位）+ 泛型 `RoomHubInner<K>`/`Room<K>`，大廳、房間 CRUD、座位、斷線、`handle_common` 指令分派全共用。各遊戲留 `engine.rs`（純邏輯）+ `hub.rs`（impl RoomKind）+ `service.rs`（開局/對局動作/廣播）。`registry.rs` 以獨立 variant（`AnyHub::Avalon` / `AnyHub::Farm`）掛載（無 watcher），統計走泛型 `summarize_room`。avalon=社交推理（私有角色推送 + 階段機 + chat）；farm=worker-placement（完全資訊，全盤狀態廣播）。**新增 N 人遊戲**：engine + impl RoomKind + service（只寫遊戲專屬指令），registry 加 variant。

## 身份識別：user vs member

| | user | member |
|---|---|---|
| 定義 | 後台管理員 | 前台會員 |
| 登入方式 | name + password 或 passkey（WebAuthn） | OAuth（Google / GitHub / LINE） |
| JWT role 欄位 | `"admin"` | `"member"` |
| middleware | `authorize_and_load` | `authorize_member` |
| struct | `AuthenticatedUser`（含 permissions） | `AuthenticatedMember`（只含 member_id） |
| DB table | `users` | `members` + `member_oauth` |
| RBAC | 有（roles / permissions） | 無 |
| token 續期 | `POST /admin/auth/refresh`（Bearer 未過期 → 新 1h token） | `POST /oauth/refresh`（jti rotation） |

**兩套系統完全獨立**，不共用 JWT、middleware、table。

### Admin passkey 登入（WebAuthn）

密碼登入的可選升級路徑（密碼永遠保留）。passkey 只替換「證明你是 user X」那步，驗證成功後走 `services/auth.rs::complete_admin_login`（密碼登入同函式）→ 同一套 JWT/middleware/RBAC。

- **實作**：`services/webauthn.rs`（ceremony）+ `repositories/passkeys.rs`（`user_passkeys` table，一 user 多把，整包序列化 `Passkey` 存 jsonb）；user handle 是 `users.webauthn_user_handle`（隨機 uuid，不用 bigint id）
- **端點**（都在 `/admin/auth` 下）：註冊 `POST /passkeys/register/begin|finish`（保護）、登入 `POST /passkeys/login/begin|finish`（公開，獨立限流 `webauthn` 10 req/60s）、`GET /passkeys` + `DELETE /passkeys/{id}`（保護，只能動自己的）
- **挑戰狀態存 Redis**：`webauthn:reg:{user_id}` / `webauthn:auth:{auth_id}`，TTL 300s，finish 時 GETDEL 一次性消費防重放
- **RP ID/origin 走 app_settings 平台保留 key**（`webauthn_rp_id` / `webauthn_rp_origin`，migration 預設 `localhost` / `http://localhost:3000` 對齊本地開發；生產/商家 instance 到 `/admin/platform` 填自己的網域，PATCH 即時生效——`Settings::reload` 重建 `Webauthn` instance，缺漏/無效 = None、passkey 端點回 500、密碼登入不受影響）。**不吃 env**。PATCH 只驗單值形狀，**刻意不驗兩 key 配對**（用另一半現值驗會讓整組換網域死鎖）；配對由前端存檔前檢查 + reload 建構失敗記 error log。**RP ID 有使用者建 passkey 後不可改**（改 = 全部作廢）
- ⚠️ **最陰的坑**：`start_passkey_registration` 預設不要求 resident key，`begin_registration` 內必須 mutate challenge 的 `resident_key = Required`，否則註冊成功但 Conditional UI（登入頁 autofill）永遠不會跳出——有單元測試守著

## 權限檢查（RBAC）

需要細粒度權限控制的 route group 用 `authorize_and_load`（不只是登入驗證）：

```rust
.layer(middleware::from_fn_with_state(state.clone(), auth::authorize_and_load))
```

handler 內用 `Perm` enum 檢查：

```rust
auth_user.require_permission(Perm::RoleRead)?;  // 無權限 → 403
```

### ⚠️ 實際權限分佈：`role_permissions` 是空的，別只讀 baseline migration

**`20260705100000_restrict_user_admin_default_role.up.sql` 把整張 `role_permissions` DELETE 掉了**（註解：「權限全面收攏給 super_admin」）。所以跑完全部 migration 後的真實狀態是：

- `guest` / `member` / `admin`（role 1/2/3）**零權限**
- `super_admin`（role 4）的全權來自**程式**而非該表
- 之後各角色權限一律由 super_admin 在後台「角色」頁（`PUT /admin/roles/{id}/permissions`）逐一勾選
- `new_user_default_roles` 設定（值為 `admin`）決定新建管理員預設掛哪個角色 —— 也就是新管理員預設是**零權限**的

**這是審查時最容易踩的坑**：baseline migration 裡有大量 `INSERT INTO role_permissions VALUES (3, 1..20)`，光讀它會以為內建 `admin` 有 `user:*`、`role:read` 等等，於是把某些路徑的可達性判斷得比實際嚴重。**要確認「誰真的能打某支端點」，一定要查 migrate 完的 DB，不要從 migration 檔推論**：

```sql
-- 某個 role 實際有哪些權限
SELECT p.resource||':'||p.action FROM role_permissions rp
  JOIN permissions p ON p.id = rp.permission_id WHERE rp.role_id = 3 ORDER BY 1;
-- 哪些 role 有某個權限
SELECT r.id, r.name FROM role_permissions rp
  JOIN permissions p ON p.id = rp.permission_id
  JOIN roles r ON r.id = rp.role_id WHERE p.resource='user' AND p.action='create';
```

推論是：目前**任何 `/admin/*` 端點實際上只有 super_admin 打得到**（除了 `/admin/auth/*` 那些自助端點）。這讓很多「低權限管理員能否越權」的疑慮在**今天**不成立，但也意味著**任何權限一旦在後台被勾給某個自訂角色，對應的越權路徑就立刻成立且不需程式改動** —— 所以 guard 該補還是要補（2026-08-01 補的 `services/roles.rs::ensure_assignable` 就是這種「先上膛的陷阱」）。

### super_admin

`super_admin` role 自動擁有所有 permissions，不需手動指派。  
實作於 `repositories/roles.rs` 的 **`get_user_permission_strings_by_id`**（早期叫 `_by_email`，已改用 user id）：偵測到 `super_admin` role 直接回傳全部 `permissions` table 內容，不看 `role_permissions`。`is_super_admin` 與權限、顯示名一起走身分快取（見下節）。

### 新增 permission

1. DB `permissions` table 插入新行（`resource`, `action`）
2. `src/structs/roles.rs` 的 `Perm` enum 加 variant + `as_str()` match arm
3. handler 用 `Perm::YourVariant`

Permission string 格式為 `"resource:action"`（如 `"role:read"`）。  
權限走下節的身分快取，`set_user_roles` / `set_role_permissions` / `delete_role` 時自動失效重載。

### ⚠️ `permissions` 表有 9 個沒有 `Perm` variant 的孤兒列

`permissions` 表目前 **46 列**，但 `Perm` enum 只有 **37 個**。差額全是 baseline migration
（`20260702000000`）帶進來、後來功能被拿掉或改名卻沒清掉的列：

`blog:create`、`note:read` / `note:create` / `note:update` / `note:delete`、
`stock:create` / `stock:delete` / `stock:manage`、`user:update`

（`invoice_lottery:write` 原本也在此列，已由 `20260911000000` 的 DELETE 清掉 —— 那是唯一清過的一個。）

**後果是靜默的**：`GET /admin/permissions`（`repositories/permissions.rs`）回整張表不做過濾，
所以後台「角色」頁的權限勾選清單**看得到這 9 個**，勾了會寫進 `role_permissions`、
也會出現在 `AuthenticatedUser.permissions` 裡，但全 repo 沒有任何 `require_permission`
比對它們 —— 等於「勾了什麼都沒發生」。

⇒ 判讀提示：看到某個角色有 `note:*` 或 `stock:manage` 不代表它能做什麼。
要清的話是**新寫一支 migration DELETE**（比照 `20260911000000`，`role_permissions`
有 ON DELETE CASCADE 會一起消失），不能改 baseline —— 已套用的 migration 動了 checksum 就起不來。

### 身分快取：一把 key 裝齊三件事（2026-08-09）

`middleware/auth.rs` 每個 `/admin/*` 請求要三個值：顯示名、`is_super_admin`、權限清單。
收斂前只有權限進 Redis，另外兩個**每個請求打一次 PG**（`users_repo::get_identity_by_id`）
—— 也就是說「快取命中」也還是省不掉 DB round-trip。三者的失效條件完全相同，沒有理由分兩份存。

- 快取內容 = `structs::auth::CachedIdentity`，key `user:identity:{id}`（前身 `user:permissions:{id}`），
  TTL `IDENTITY_TTL_SECS` 1 小時。
- **唯一載入點是 `services::auth::load_identity`**：命中＝零 DB；miss 才查 DB，兩支查詢
  （identity + permissions）以 `try_join!` 併發，回寫失敗只記 WARN 不擋請求。
  回 `None` = 帳號已刪（token/session 未過期）→ 視為未授權。
- 呼叫端兩個：middleware 與 `services::torrents::resolve_download_file`（簽名連結即時重查
  發行者權限）。**收斂前兩邊各有一份逐字相同的 Redis→DB fallback。**
- 登入（`complete_admin_login`）先 `invalidate` 再 `load_identity` 預熱：登入是「重新確認
  這個人是誰」的時點，不該沿用上一輪快取；載入順帶寫回，登入後第一個請求就不查 DB。
- ⚠️ **`name` 能被快取的前提是 `users.name` 不可變**（全 repo 只有 password 有 `UPDATE users`）。
  哪天加了改名端點，那支必須呼叫 `redis::invalidate_user_identity`。
- 快取值解不開（換過形狀的舊值）**視同 miss**，不讓一顆壞 key 把整個 `/admin/*` 打掛（附測試）。

## 新增 route 模組

1. 建立 `src/routes/<name>.rs`，定義 `pub fn new(state: AppState) -> Router<AppState>`
2. 在 `routes.rs` 加 `mod <name>;`
3. 在 `app()` 加 `.nest("/<path>", <name>::new(state.clone()))`
4. 需要 JWT 驗證的 router 用 `super::with_auth(state, router)` 套上 middleware：

```rust
// 全部保護
pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(handler)))
}

// 公開 + 保護混用
pub fn new(state: AppState) -> Router<AppState> {
    let protected = super::with_auth(
        state,
        Router::new().route("/protected", get(protected_handler)),
    );
    Router::new().route("/public", get(public_handler)).merge(protected)
}
```

### 分層鐵律：`routes/` 不得 import `repositories/`（2026-08-09 收齊）

`routes → services → repositories` 是單向的，**route 一律只呼叫 service**。目前
**由 CI 強制**（2026-09-24）：`scripts/check-layering.sh` 在 backend test job 跑，違反即紅燈；同一支腳本也守 `repositories/ ↛ services/routes`、`structs/ ↛ 三層`。`jobs/` 可直呼 repositories（排程沒有 route 那種「邏輯就地長在呼叫端」的問題），不在檢查範圍。

在此之前有 13 處破例，兩類：

1. **薄讀取直呼 repository**（`metrics` / `logs` / `admin_stats` / `admin_vocab` / `auth` 的
   passkeys / `ws` 的 ticket + 到訪採集 / `torrents` 的 list）。看起來只是少一層轉呼，
   但邏輯會就地長在 route 上 —— `admin_stats` 的三支查詢就這樣在 route 裡序列 `await`
   了很久，搬進 `services/stats.rs` 後順手改成併發。
2. **擁有者檢查散在 route**（`images` / `blogs` / `blog_comments` / `torrents` ×4）：
   每支端點自己先 `repo::get_owner(...)` 再 `require_owner`。**這類最危險** —— 資料隔離
   規則寫在呼叫端，漏掉一支不會有任何徵兆，只會安靜多一個能讀他人資料的側門
   （`admin_blog_comments` 的那道就是後來補的）。現在檢查與寫入綁在同一支 service
   函式裡（`delete_image(pool, storage, actor, id)` 這種形狀），漏不掉。
   - torrents 的 4 支共用私有 `ensure_owner`；排程 job 走不做檢查的 `delete_by_id`。

**API 回應型別同理搬進 `structs/`**（`Log` / `AuditLog` / `ImageRecord` / `SystemMetric` /
`DailyVisitorStat`）—— 型別留在 `repositories/` 的話 route 還是得 import 它。

member 端（前台會員）用 `super::with_member_auth(state, router)`，**不要直接掛 `auth::authorize_member`** —— 直接掛會跳過 audit 層，那正是 `/member/*` 長年沒有稽核紀錄的原因。訪客也能用的端點（`authorize_member_optional`，目前 `routes/vocab.rs` 與 `routes/blogs.rs` 的留言）不在此列。

**現存的唯一例外是 `routes/members.rs` 的 `GET /members/me`**（同檔的 admin 兩支走 `with_auth`，member 那半直接掛 `authorize_member`）：member 的稽核**只記非 GET**，所以這支掛不掛 audit 層都不會產生任何一列。要在那個 router 上加任何非 GET 端點時，必須先改成 `with_member_auth`。

## 新增排程 job

1. 建立 `src/jobs/<name>.rs`，定義 `pub async fn run(state: AppState)`
2. 在 `jobs.rs` 加 `pub mod <name>;`
3. 在 `structs/jobs.rs` 的 `AppJob` enum 加 variant，補 **4 個** match arm：`name()`（scheduler log 用）、`feature()`（功能開關，core job 回 `None`）、`cron_expression()`、`run()`
4. 在 `structs/jobs.rs` 的 `AppJob::ALL` 加入新 variant（`scheduler.rs` 從這個常數迭代，**不要**在 scheduler 另抄一份清單）
5. 需要重試邏輯的 job 用 `super::run_with_retries(label, max_attempts, retry_delay, || fut)`：

```rust
pub async fn run(state: AppState) {
    let pool = state.get_pool().clone();
    let client = state.get_http_client().clone();
    super::run_with_retries(
        "my_service",
        3,
        std::time::Duration::from_secs(3600),
        || my_service(&pool, &client),
    )
    .await;
}
```

## 儲存層與圖片上傳

`Storage::from_env()` 讀取 `UPLOAD_PATH`。`upload(data: &[u8], ext, base_url)` 只管落地已處理完的 bytes（`upload_base_url` 從 `app_settings` 即時讀取,由 caller 傳入）。  
新增後端：在 `Storage` enum 加 variant，並在 `upload` / `delete` match 補上對應分支。  
靜態檔案：圖片公開網域一律為 `media.kawa.homes`（nginx 直出 `/srv/kawa/uploads`，見 `deploy/nginx/conf.d/media.kawa.homes.conf`），`upload_base_url` = `https://media.kawa.homes`。**存量舊圖已於 2026-07-28 全數回填**（migration `20260728000000_media_domain_backfill`：`images.url` + `blogs.markdown` 字串替換，同一 tx；兩網域指向同一磁碟目錄故零檔案搬動），DB 內不再有 `axum.kawa.homes/uploads/...`，該路徑在 nginx 已改成 301 到 media，只為接 repo 外改不到的舊連結。backend `GET /uploads/*` → `ServeDir` **保留但生產不會被打到**：那是本地開發（無 nginx）唯一能瀏覽剛上傳圖片的路徑，本地請把 `upload_base_url` 設成 `http://127.0.0.1:3000/uploads`。

圖片驗證/轉檔在 `services/images.rs::process_image`（storage 不做內容驗證）:
- 用 `image` crate 實際 decode 驗證「真的是圖片」,非圖片/損毀/超限回 400,不落磁碟
- 上限:單邊 16383px（= libwebp 上限）、總像素 40MP、decode alloc 128MB — 三者都是擋 decode-bomb / OOM（1核1G VPS）
- 統一轉 lossy WebP（`webp` crate / libwebp）,順帶剝除 EXIF;GIF 例外:驗證後保留原檔（避免動畫被壓成第一幀）。品質從 `app_settings.image_webp_quality`（1–100）即時讀,route 讀值 clamp 後傳入 `process_image(data, quality)`,缺失/壞值 fallback `DEFAULT_WEBP_QUALITY`=80
- CPU 密集,caller 一律包 `spawn_blocking`（1 核機上同步跑會佔住 async worker,慣例同 auth.rs 的 bcrypt）。目前的 `spawn_blocking` 點:`services/auth.rs` 的 bcrypt 三處、`services/images.rs::process_image`、`services/tools.rs::convert_text` 的 zhconv（2026-07-31 補，2026-08-31 由 route 搬進 service）。**公開端點另外要限輸入長度**——全域 body 上限是 10MB,對單筆計算來說過寬;`convert_text` 限 256KB（實測 zhconv 0.4.1 的 CPU 成本不高:10MB≈28ms、1MB≈2.5ms,所以那個上限主要是擋 1 核 1G 機器上的記憶體,不是 CPU）。**還沒包的**:`services/stocks.rs::parse_buyback_stock_raw_html` 的 regex 走訪（只在排程時發生、非使用者可觸發,故暫留;2026-08-19 前是 `scraper` 的 `Html::parse_document`）
- multipart 只處理有 `file_name` 的欄位,文字欄位跳過

## 環境變數

完整表格見 `README.md`「環境變數」一節（進版控，是唯一可信來源；改 env 要同步改那份）。必填只有 `DATABASE_URL` / `REDIS_URL` / `JWT_SECRET`。

**`REDIS_URL` 是 2026-08-01 從 `REDIS_HOST` 改過來的**（舊寫法把 6379 寫死在 `state.rs`，要給 valkey 加密碼就得改 Rust）。同日 VPS 的 `/srv/kawa/env/kawa.env` 已加上 `REDIS_URL=redis://valkey:6379`，故過渡期的 `REDIS_HOST` 回退路徑已移除 —— **`REDIS_HOST` 現在完全沒有作用**，只認 `repositories/redis.rs::redis_url_from_env` 讀的 `REDIS_URL`。

**`APP_PORT` 在生產是假旋鈕**：3000 被 `deploy/nginx/conf.d/02-proxy.conf` 的 upstream 與 `kawa.env` 的 `API_URL` 寫死，只改 env 會讓 nginx 照打 3000 → 502。故 `deploy/env.example/kawa.env` 刻意不放這個 key，它只服務本機直跑（3000 被佔時換一個）。

## 本機端到端驗證（起一套拋棄式環境）

`cargo test` 只有純函式測試，任何「這支端點真的擋住了嗎」都得起真的服務。本機有 docker，配方如下（2026-08-01 實跑過）：

```bash
# 1. DB + Redis。valkey 映射到哪個 port 都行，REDIS_URL 帶得動（2026-08-01 起；
#    在那之前 6379 寫死在 state.rs，只能映 6379）。
# ⚠️ WSL 上 55432 可能落在 Windows 的保留 port 範圍（`bind: An attempt was made to access a socket
#    in a way forbidden`），換一個（如 15432）並同步改下面的 DATABASE_URL。
docker run -d --name verify-pg -e POSTGRES_DB=kawa -e POSTGRES_USER=kawa \
  -e POSTGRES_PASSWORD=verify -p 55432:5432 postgres:18-alpine
docker run -d --name verify-valkey -p 6379:6379 valkey/valkey:alpine

# 2. 起 backend。migration 由 routes.rs 在啟動時自動跑（sqlx::migrate!），不需 sqlx-cli。
#    監聽 port 預設 3000（`main.rs` 的 `APP_PORT`）；本機 3000 被佔時帶 APP_PORT=xxxx 換一個，
#    生產不能改（nginx upstream 寫死 3000，見「環境變數」）。
DATABASE_URL=postgresql://kawa:verify@127.0.0.1:55432/kawa REDIS_URL=redis://127.0.0.1:6379 \
  JWT_SECRET=$(openssl rand -hex 32) TRUST_CF_HEADER=true \
  UPLOAD_PATH=/tmp/vu TORRENT_PATH=/tmp/vt cargo run
curl -fsS http://127.0.0.1:3000/health
```

要 admin 身分時：直接往 `users` 塞一筆（bcrypt hash 可用 `docker run --rm python:3-alpine sh -c "pip install -q bcrypt && python -c '...'"` 生），再 `INSERT INTO user_roles (user_id, role_id) VALUES (<id>, 4)` 給 super_admin，然後 `POST /admin/auth` 拿 token。member 身分更省事：用同一把 `JWT_SECRET` 自己簽一個 `{sub, role:"member", exp, iat}`。

**nginx 設定驗證**照 `.github/workflows/deploy.yml` 的 `validate-nginx` job 抄（自簽憑證 + `--add-host backend/frontend:127.0.0.1`）。但 `nginx -t` 只驗語法；**要驗語意（例如新增 location 有沒有讓 server 層的 `add_header` 整組掉光）必須真的起 nginx 去 curl 看 response header** —— 後端打不通會回 502，但 header 是 nginx 加的，照樣看得到。`limit_req` 也能這樣實測（連打 N 次看第幾次變 429）。

用完 `docker rm -f verify-pg verify-valkey`。

## 主要依賴

完整清單見 `README.md`「技術棧」一節（新增/移除依賴要同步改那份）。librqbit 用 `default-features = false` + `rust-tls`，見下方依賴約束。

### 依賴約束：Debian builder + distroless/cc（2026-07 起，前身是 alpine/musl → scratch）

Docker build 是 `rust:bookworm`（glibc 動態連結）→ `gcr.io/distroless/cc-debian12`（含 glibc/libgcc/tzdata/ca-certs，無 shell）。builder 與基底必須同為 Debian 12,glibc 版本才對齊。

- 已無 musl / 禁 openssl 的硬限制;既有依賴仍走 rustls 類 feature（`rustls-tls` / `rust-tls`），無必要不引入 openssl
- **`librqbit` 的 `rust-tls` 要留著,但理由已不是 musl**（2026-07 查證）。該 feature 綁了兩件不相干的事:`reqwest/rustls-tls`（要的）＋ `sha1w/sha1-ring`（SHA-1 後端,名字叫 ring 但實際是 `aws-lc-rs` → `aws-lc-sys`），librqbit 沒給拆開的辦法。另一邊 `default-tls` 會換成 `crypto-hash` 0.3（近乎停更）＋ native-tls，**更差**,所以維持現狀。aws-lc-sys 的 RUSTSEC 公告（X.509 name constraint / PKCS7_verify / AES-CCM timing / CRL）走的是憑證驗證路徑,**這裡只拿來算 SHA-1,不觸發**;版本仍應跟上（已升 0.43.0）
- **例外:webauthn-rs 硬依賴 OpenSSL ≥3.0**,故 openssl crate 開 `vendored` feature（隨原始碼編譯靜態連結,不吃系統/基底 image 的 libssl;本機 WSL 1.1.1 也能編）
- C 依賴可用:aws-lc-sys（librqbit）0.43 起走 **cc builder,不需要 cmake**（2026-08-28 已從 Dockerfile 移除那行 apt-get;證據:本機 WSL 無 cmake 卻編出 `target/release/build/aws-lc-sys-*/out/libaws_lc_0_43_0_crypto.a`,同一個 target triple `x86_64-unknown-linux-gnu`。若哪天 build 掛在 aws-lc-sys 找不到 cmake,把那行加回來即可）、libwebp-sys（webp crate）走 cc、vendored openssl 需 perl/make（基底內建 gcc/make/perl）
- 最終 image 無 shell,`docker exec` 進不去;debug 靠 log

## 政府採購網標案追蹤（gov_tenders）

依 `app_settings.gov_tender_keywords`（逗號分隔）每日抓政府電子採購網標案公告，落地 `gov_tenders` table，後台授權查閱。

- **資料來源**：g0v 社群 API `https://pcc-api.openfun.app/api/searchbytitle`（舊網址 `pcc.g0v.ronny.tw` 已轉址；非官方 SLA，job 有重試）。每關鍵字抓第 1 頁 100 筆（依公告日新到舊），足以涵蓋每日增量。
- **排程**：`FetchGovTenders`（每日 UTC 23:00 = 台北 07:00，`jobs/fetch_gov_tenders.rs`）：抓取 → `ON CONFLICT (filename) DO NOTHING` 去重寫入 → 新公告寄 email 通知（走 `notification` 類 SMTP 設定的 `send_notification`，收件人 `notify_email`）。
- **通知規則**：`notified_at` 欄位追蹤；**首次執行**（表為空）只建存量、標記不寄信；SMTP 未設定則不標記、待設定後下輪補寄；信最多列 50 筆。
- **官方連結**：`detail_url` 由公告日 + 檔名組成 `https://web.pcc.gov.tw/prkms/tender/common/noticeDate/redirectPublic?ds=<YYYYMMDD>&fn=<filename>.xml`（303 轉官方公告頁）。
- **查詢**：`GET /admin/gov_tenders?keyword=&tender_type=&q=&page=&per_page=`（需 `gov_tender:read`，`q` 為標案名稱/機關 ILIKE、`tender_type` 完全比對），回 `{ data, total }`；`GET /admin/gov_tenders/types` 回所有出現過的類型（去重排序 `Vec<String>`，供前端篩選下拉）。
- 解析純函式 + 測試在 `services/gov_tenders.rs`（`parse_records` / `parse_date`，單筆欄位缺漏跳過該筆不炸整批）。

## 不重複到訪統計（visitor stats）

「每日不重複到訪人數」。**採集點 = WS 握手成功時**（前台頁載入即連 WS，天然濾掉不跑 JS 的 bot/爬蟲），去重元素 = `ip|ua`（IP 取自 CF-Connecting-IP，fallback socket）。

- **計數走 Redis HyperLogLog**：key `visitors:<台北日 YYYY-MM-DD>`，當日首寫設 40 天 TTL。元素丟進 HLL、Redis 不存原始 IP/UA。日界以**台北時間（UTC+8）**為準。
- **採集**：`routes/ws.rs` 的握手 handler spawn `services::stats::record_visit`（→ `repositories::visitors::record_visit`）（best-effort，不阻塞連線、失敗只 warn）。
- **落地**：`jobs/aggregate_visitors.rs`（每日 UTC 16:05 = 台北 00:05）把前一台北日 `PFCOUNT` upsert 進 `daily_visitor_stats(date, unique_visitors)`。即時當日值仍直接讀 Redis。
- **查詢**：`GET /admin/stats/visitors?days=30`（需 `stat:read`）→ `today`（即時 PFCOUNT）+ `last_n_days_unique`（近 N 天 PFCOUNT 多 key 跨日去重）+ `history`（DB 歷史，新到舊）。
- repository：`repositories/visitors.rs`（`record_visit` / `count_day` / `count_days` / `upsert_daily` / `history`；台北日界走 `utils/date::taipei_today`，本檔已不再自帶那支）。

## 應用日誌（logs）

`logging.rs` 的 `DbLogLayer` 把 **WARN / ERROR** 落地 `logs` table（批次 50 筆或 500ms flush，保留 14 天由 `jobs/cleanup_observability.rs` 清）。業務事件走另一套 `admin_audit_logs`（audit middleware），不在此表。

### 兩層門檻：`RUST_LOG` 是天花板，`log_db_level` 在底下調（2026-08-08）

- **stdout 層**：`main.rs::default_log_filter()`。未設 `RUST_LOG` 時，release（生產與所有商家 instance）= `info,tower_http=warn`、debug build（`cargo run`）= 全 debug。**刻意寫在程式裡不放 `kawa.env`** —— 那個檔在 VPS 上、不進版控，等於把生產行為藏在 review 不到也 diff 不到的地方，新 instance 還得記得再設一次。`RUST_LOG` 退成臨時逃生門。
  - `tower_http=warn` 掐掉的是 `on_request`/`on_response` 那兩行 DEBUG（**每個請求兩行**，而 compose 給 backend 的 rotation 是「稀疏」規格 10m×3）。`on_failure` 是 ERROR，5xx 那第三筆照樣留著。
  - ⚠️ `dotenvy::dotenv()` **必須在 tracing 初始化之前**：`try_from_default_env()` 讀的是真正的 process env。2026-08-08 之前順序是反的，所以 `backend/.env` 裡的 `RUST_LOG` 靜默失效，只有 `export` 走得通。
- **DB 層**：`app_settings.log_db_level`（ERROR / WARN / INFO），值住在 `logging.rs` 的 `static DB_LEVEL`（process 全域 —— subscriber 本來就是，而且它得在 `AppState` 之前初始化，為它把 `Arc` 穿過四層建構子換不到東西），`Settings::reload` 推過去。
- ⚠️ **兩層相依**：`EnvFilter` 掛在 registry 上是**全域** filter，被它擋掉的 event 根本到不了 `DbLogLayer`。所以 stdout 那層設 `info` 時，`log_db_level` 的可用上限就是 INFO；哪天把它收成 `warn`，後台那個旋鈕就變成無聲失效的裝飾品。
- **刻意不開 DEBUG**（`DB_LEVEL_VALUES` 只有三個值，PATCH 擋下）：留在 debug 的是 404 與過期 token 這兩類噪音，開下去等於一隻掃描器打一輪就把 PG 灌爆。要看 DEBUG 只能調 `RUST_LOG` 走 stdout。
- 用法：線上要追一段沒有 WARN 的行為時把它調到 `INFO`，用 `kawa-logs` 查完調回 `WARN`。INFO 收得到 job 成敗、登入成功、torrent 生命週期這些「沒壞但想知道」的事件。

### 分級判準：只有「要人看一眼」的才配 WARN

2026-08-08 用生產實測資料重訂（當時 14 天內 89 筆 WARN+，其中 **76% 是兩條噪音**）：

- **WS 收訊/送出失敗記 debug 不記 warn**（`services/ws/socket.rs`、`state.rs` 的 `send_many`/`broadcast`）：40 筆全是 `Connection reset without closing handshake` —— 關分頁、手機睡眠、NAT 逾時都會產生，是公開網站的常態，而且斷線後清理照常走完，沒有任何要人介入的事。
- **`AuthError` 分兩級**（`errors.rs::is_routine_auth`）：`MissingToken` / `TokenExpired` / `InvalidToken` / `InvalidHeader` / `Unauthorized` → debug（前端 token 只有 1 小時、`kawa-logs` CLI 也是 401 才續期，這是客戶端常態）；`Forbidden` / `InvalidPassword` / `InvalidCredentials` / `WebauthnFailed` / `UserNotFound` → warn（帶著身分卻被擋下，每一筆都值得看，且能用 `request_id` 對回 `admin_audit_logs` 查是誰）。
- **`RequestError` 同樣分兩級**（`errors.rs::is_routine_request`，2026-08-09）：`UnprocessableContent` / `Conflict` / `InsufficientStorage` / `MultipartError` / `InvalidContent` / `Rejection` → **warn**；`NotFound`、`TooManyRequests` 與 `PathRejection` 留 debug。在那之前整個 `RequestError` 都是 debug，而落地門檻是 WARN，於是**所有 4xx 在 `logs` 表零紀錄** —— 使用者回報最多的「按了沒反應 / 存不進去」正是 422 與 409。
  - `NotFound` 的理由同 `TokenExpired`：爬蟲掃站與 `with_feature` 關閉功能都走這條。
  - ⚠️ **`TooManyRequests` 刻意不提上來**：被擋下的請求本來就是連續一整串，每筆一列等於讓攻擊者決定 `logs` 表的寫入量（實測連打 25 次會落 5 列）。
- **限流 429 記 warn**（`middleware/rate_limit.rs`）：承上，429 的訊號全靠這則 —— 只在 `count == max + 1`（剛超過）那一刻記一筆，帶 scope 與 ip，持續打不會每筆寫一列。
- **`logs` 表自己的落地失敗走 `eprintln!`**（`logging.rs` 的 `report_dropped` / `flush`）：不能用 `tracing`（會從 `on_event` 遞迴回 `on_event`，而且壞掉的正是那條路）。丟棄數累加在 `static DROPPED`、由 `log_writer` 每 500ms 匯總印一行，DB insert 失敗直接印。吞掉的話「logs 表停止寫入」與「真的沒事發生」長得一模一樣。

### access log 與 panic（2026-08-09 補）

- **每個請求都有一行 INFO 收尾**：`routes.rs` 的 `TraceLayer::on_response` 記 `status` + `latency_ms`，target 是 **`api_server_axum::access`**（`logging::ACCESS_TARGET`）。在那之前 `logs` 表只有出錯時才有紀錄，`GET /logs/request/{id}` 對一個成功請求回空陣列 —— 答不出「這個 request 回什麼、跑多久」，追「變慢」或「回 200 但資料錯」完全沒有時間軸。
  - **target 刻意長在 crate 名底下**：EnvFilter 的 directive 是前綴比對，所以 `api_server_axum=info` 自動涵蓋它，`default_log_filter()` 不必為它多列一段；要單獨靜音仍可 `RUST_LOG=api_server_axum=info,api_server_axum::access=off`（它是每請求一行，量級跟其他 INFO 差一級）。⚠️ 同日一度用過裸 target `http_access`，那要求預設 filter 額外列一段，而任何人手動設 `RUST_LOG` 漏了那段，access log 就整條靜默消失 —— **新增 target 一律掛在 crate 名底下**。
  - 平時只在 stdout；`log_db_level=INFO` 時才進 `logs` 表 —— 要抓一段完整軌跡就臨時調上去，查完調回 WARN。
- **span 帶 `query` 與 `ip`**（原本只有 method / path / request_id）：query 讓分頁/篩選類 bug 能重現參數，ip 讓限流與 abuse 追得到來源（IP 判斷與限流、到訪統計共用 `utils::net::client_ip`）。**query 進 log 前先過 `utils::redact::redact_query`** —— `/ws?ticket=` 與 oauth callback 的 `?code=` 都是憑證，本檔「WS 連線的 admin 身分」那條「JWT 不走 WS query，因為會進 access log」的規定就是在這裡兌現的。
- **所有 panic → 一筆 ERROR（含檔案:行號）**（2026-09-07）：`main.rs::install_panic_hook` 在 subscriber 之後裝一個包住預設 hook 的 panic hook（預設 hook 先跑，backtrace 只有它拿得到），payload 走 `errors::panic_message`（與 `CatchPanicLayer` 共用）。**在此之前只有請求路徑與排程有接住**，那之外的裸 `tokio::spawn` 全部無痕：torrents 的 `sync_active` / `run_torrent`、ws 的 recv/ping task、遊戲的 `timeout_watcher`、`state.rs` 的 per-connection 廣播 task，以及**兩個批次寫入器自己**。handler panic 因此會有**兩筆**（hook 那筆帶 location、`handle_panic` 那筆帶 request_id，同 span 故同 request_id）。⚠️ hook 內有 thread-local 遞迴防護：`DbLogLayer::on_event` 自己 panic 時不再走 `tracing`（否則 panic in panic → abort），只留預設 hook 的 stderr。
- **批次寫入器的死亡會被喊出來**（`batch_writer::supervise`，`routes.rs` 對兩個 writer 各掛一個）：`log_writer` panic 之後 tracing 這條路本身就斷了（event 照樣進 channel，但沒人取出，連 `report_dropped` 都隨那個 task 一起沒了），所以用 `eprintln!` —— 那是管線壞掉時唯一還活著的通道。**正常關機不會印**（`DbLogLayer` 的 sender 活在全域 subscriber 裡永不 drop，process 先退場），這條存在的意義就是 panic 那支。
- **handler panic → 500 + ERROR**（`errors.rs::handle_panic` 掛 `CatchPanicLayer`，**在 TraceLayer 內層**，這樣 span 與 request_id 的 task-local 都還在）。沒有這層的話 panic 只會切斷連線：client 看到 network error 而不是 500、`logs` 表零筆、稽核也不會有那筆（audit 在 `next.run` 之後才寫），唯一痕跡是 runtime 預設 hook 印在 stderr、不進 tracing 的那行。job 那側 2026-08 就補過同樣的防護，請求路徑一直沒有。

- **`request_id` 是查線上問題的主入口**。`routes.rs` 那條 request span 帶 `request_id` / `method` / `path` / `query` / `ip`，`DbLogLayer::on_new_span` 把 span field 存進 span extensions、`on_event` 走 `ctx.event_scope()` 由內而外併回來（**tracing 不會自己保留 span field，少了 `on_new_span` 這步 DB 裡的 log 就對不到任何請求**）。`request_id` 升成獨立欄位＋partial index，其餘進 `fields jsonb`。span 填的 `"-"` 視同沒有。
- **錯誤細節在 `fields->>'self'`**，不在 `message`。`tracing::error!(?self, "System error occurred")` 的 message 只是那句固定字串，真正的原因（`SystemError(Redis(...))`）是名為 `self` 的 field —— 所以 `?q=` 會同時掃 `message` 與 `fields::text`，只搜 message 找不到有用的東西。單一 field 值截斷在 `MAX_FIELD_LEN`(2000 字)。
- 一個 5xx 通常落 **3 筆同 `request_id`**：來源處的 error（如 `state.rs` 的 Redis 失敗）＋ `errors.rs` 的 `System error occurred`（帶 `self`）＋ `tower_http::trace::on_failure` 的 `response failed`（帶 latency / classification）。5xx 不必另外補 log，TraceLayer 預設的 `on_failure` 已經是 ERROR。
- **查詢**：`GET /logs`（需 `log:read`）吃 `level`（逗號分隔多值，大小寫不敏感）/ `q` / `target` / `request_id` / `from` / `to` ＋分頁，回 `Paginated<Log>`＝`{ data, total }`；`GET /logs/request/{request_id}` 回該請求完整軌跡（**時間正序**，與列表的新到舊刻意相反，上限 500 筆，刻意不分頁故回裸陣列）。`q` / `target` 的 ILIKE 無索引，量大時先用 `from`/`to` 收範圍（`created_at` 有索引）。
- **篩選條件只有一份**：`repositories/logs.rs` 的 `const LOG_FILTER`（$1..$6）被 `get_logs` 與 `count_logs` 共用，**兩邊 bind 順序必須一致**，加參數要同時改。條件寫兩份就是 `total` 與 `data` 對不上的來源。service 層用 `tokio::try_join!` 併發跑 count 與 list。
- 前端 `api/logs.ts` 回整包不解包（2026-08-03 起全站 api 層統一如此），由消費端在 `usePagedList` 的 fetcher 閉包內取 `.data`，**改回應形狀要同步那支**。

### 從本機查 production log:`scripts/kawa-logs`

`scripts/kawa-logs`（python3 stdlib，無外部依賴）走 HTTPS 打 `api.kawa.homes`，子命令 `rid` / `errors` / `logs` / `metrics` / `audit`，輸出 JSONL。設定檔 `~/.config/kawa/config`（**不進 repo**，600）。

- **身分是後台的 `log_reader` 帳號**，只掛 `log:read` / `metric:read` / `audit:read` 三個權限、非 super_admin。**刻意不做 api_keys 表 / `X-Api-Key` middleware** —— 既有 RBAC 已足夠，撤銷走停用帳號或取消角色勾選，零後端程式碼。2026-08-03 實測:三支該通的 200，其餘 14 支 `/admin/*` 與 `DELETE /admin/blogs/{id}`、`POST /ws/ticket` 一律 403。
- 這是**這套部署上第一個真的被賦予權限的自訂角色**（其餘角色的 `role_permissions` 仍是空的，見上面那個坑）。
- JWT 只有 1 小時 → CLI 快取 token 在 `~/.cache/kawa/token`，401 自動重登。
- ⚠️ **Cloudflare 會把 `Python-urllib/3.x` 這個預設 UA 擋成 `403` + body `error code: 1010`**，那是 CF 回的、根本沒到 backend，但長得跟權限不足一模一樣。所以 CLI 必須自帶 `User-Agent`（curl 的預設 UA 反而過得去）。**在這個網域上看到 403，先看 body 是不是 1010 再懷疑權限。**
- ⚠️ **每次查詢都會在 `admin_audit_logs` 留一筆** —— audit middleware 掛在 `with_auth` 內層，**admin 不分讀寫、GET 也記**（member 那側 2026-08-09 起只記非 GET，見「操作稽核」）。`log_reader` 是 admin 帳號，所以每一發唯讀查詢都算。別把這支寫成高頻輪詢。這也是 2026-08-03 給 `cleanup_observability` 補 `admin_audit_logs` 180 天保留期的動因（導入時 7228 筆 / 最舊 83 天，故當下刪 0 筆）。
- 服務掛掉、API 也打不通時查不了 —— 那種情況得上 VPS 直接 `docker exec database psql`（`docker exec backend` 進不去，distroless 無 shell，只能 `docker logs backend`）。
- 前端 `app/admin/(main)/logs/` 已能顯示細節（2026-08-09）：每列一顆「展開」，開了看得到 `request_id` 與整包 `fields`（`self` 固定排第一，那才是真正的原因），並可按「整條軌跡」打 `GET /logs/request/{id}` 就地列出同一請求的全部紀錄。另有關鍵字搜尋，打的是後端的 `q`（**同時掃 message 與 `fields::text`** —— 只搜 message 找不到有用的東西）。
  - 在此之前後台只看得到那句固定 message，細節得自己走 API 或 `kawa-logs`。2026-08-09 追 member 登入失敗時就卡在這裡：頁面上只有 `Connection error occurred`，而答案（`www.googleapis.com` + `Cannot assign requested address`）整段在撈不到的 `fields.self` 裡。

## 持股損益（portfolio）

`GET /member/portfolio/summary` 與 `/{id}/history` 走三層快取管線（Redis → `stock_closing_prices` / `stock_ex_rights` → TWSE，抓到就回寫下層）。

- **上游抓取有預算，逾算就回空**（`UpstreamBudget`，2026-08-06 加）：單次請求最多 `MAX_UPSTREAM_FETCHES`(6) 次 TWSE、`UPSTREAM_TIME_BUDGET`(8 秒)，summary 的多筆持股**共用同一份預算**（clone 共享 Arc 計數與 deadline）。沒有它的話：`fetch_all_closing_prices` 是逐月抓、`services/twse.rs` 是全域 `semaphore(1)`，一個三年前買入、持股十檔的 member 按一次 summary 就是 ~360 次序列上游請求（每次 timeout 30 秒）—— 該請求撐不到回應，還會把 TWSE 通道從排程 job 手上整段搶走。
- **代價寫明**：逾預算的月份當成沒資料。已抓到的都寫進了 `stock_closing_prices`，下次請求從 DB 命中並接著補，幾次之後就完整 —— 即「剛加入的舊持股，歷史圖要多按幾次才長齊」。
- **DB 那面也有上限**（2026-08-22 加）：summary 的持股改用 `stream::buffered(SUMMARY_CONCURRENCY = 4)`，不再 `try_join_all` 無界 fan-out —— 每筆持股要跑收盤價 + 除權息兩個查詢，20 檔就是瞬間 40 個查詢搶 PG 連線池，`acquire_timeout(3s)` 一到整個請求 5xx。`buffered` 是「併發執行、依序產出」，回傳順序仍是持股清單順序。股名另外一次 `get_stock_names_by_codes`（`DISTINCT ON` + `code = ANY`，走既有的 `(stock_code, trade_date DESC)` 索引）批次取，原本是每檔一發。
- ⚠️ **月份是由新到舊抓**：預算有限時額度要先花在最新的月（summary 的現價、history 最右端都取 `closes.last()`）。由舊到新會把額度耗在最舊的月份、反而讓現價變 `None`。最後統一 `sort_by_key`，順序對呼叫端不可見 —— **改這個迴圈時不要「順手」改回正序**。
- **summary 另回三個期間的增減**（2026-09-08）：`PortfolioSummaryEntry.changes` = `{ day, week, month }`，每個是 `PeriodChange`（`base_date` / `base_close` / `change` / `change_pct` / `value_change`）或 `null`。目標日**用日曆算**（「近一週」＝七天前，不是七個交易日前），基準日取**最後一個不晚於目標日的交易日**（休市自然往前落）；期間內的除權息會還原到基準價上，否則除息日會被算成一次大跌。基準比目標日早超過 `MAX_BASE_LOOKBACK_DAYS`(10) 就回 `None` —— 上游預算逾額的月份是整段沒資料，拿三個月前的價格謊稱「近一週」比顯示「-」更糟。還原因子是三處共用的 `ex_adjust_factor`（`compute_latest` / `build_history` / `period_change`），改公式只改那一支。前端型別在 `frontend/types/portfolio.ts`，期間切換是**純前端**（`portfolio/period-tabs.tsx`，歷史表格也用它收斂區間），端點沒有 `period` 參數。
- ⚠️ **除權息的預算檢查必須在 `upsert_ex_rights_checked` 之前 return**：那筆記錄代表「已向 TWSE 確認過這 30 天沒有除權息」，沒真的問就寫等於騙自己 30 天。

## 操作稽核（admin_audit_logs）

`middleware/audit.rs` 掛在 `with_auth` **內層**，對每個帶有效 admin 身分的請求記一筆（**不分讀寫，GET 也記**），180 天保留期由 `jobs/cleanup_observability.rs` 清。查詢走 `GET /admin/audit_logs`（需 `audit:read`）。

- **也記 member 的寫入**（2026-08-09）：`/member/portfolio` 改走 `routes.rs::with_member_auth`（= `authorize_member` + 同一支 audit middleware）。在那之前 `/member/*` 直接掛 `authorize_member`、完全跳過 audit，「會員改了什麼、刪了什麼」零紀錄。
  - `actor_type` 欄（`admin` / `member`，migration `20260809000000`，舊列 DEFAULT `admin`）區分身分；`user_email` 對 admin 是顯示名、對 member 是 **`member#{id}`** —— member 的名字要多打一次 DB，稽核不值得在請求路徑上加一次查詢。`GET /admin/audit_logs?actor_type=` 可篩。
  - **member 只記非 GET**：會員讀自己的資料是常態，全記等於用 180 天保留期的表存瀏覽軌跡。admin 維持不分讀寫（那邊「誰查了會員個資」本身就是要稽核的事）。
  - **`vocab` 刻意不掛**：每答一題就是一個 `POST /runs/{id}/answer`，稽核價值近乎零而量最大。

- **寫入路徑不碰 DB**：middleware 只組 `AuditEntry` 丟進 `state.get_audit_tx()`（`try_send`），由 `services/audit_logs.rs::audit_writer` 批次寫（UNNEST 一次 INSERT）。**攢批的節奏與 `logging.rs::log_writer` 共用 `batch_writer.rs`**（容量 1000 / 50 筆 / 500ms，常數只有一份）—— 2026-08-09 前是兩份同構的 `select!` 迴圈各帶三個常數，改一邊忘另一邊不會有任何提示。各自保留的只有 `flush`（寫什麼、寫失敗怎麼辦）：log 那邊的失敗**不能**用 `tracing` 回報，會遞迴回 `on_event`。2026-08-06 前是**每請求 spawn 一個 task 各自 INSERT**，尖峰時稽核寫入會跟真正的查詢搶那 20 條 PG 連線，而稽核完全不需要即時可見。
- **佇列滿只記 `debug!`**（丟棄該筆）：這條路徑上每丟一筆記一則 WARN 等於「用寫進 logs 表來反應 DB 已經跟不上」。寫入器被關閉才記 `error!`（那是 bug，之後稽核會靜默消失）。
- **`request_id` 欄（2026-08-06 加）**讓稽核與 `logs` 表能互查。⚠️ **這個值必須在 middleware 內取**：`request_id` 是 task-local，離開請求的 task 就讀不到 —— 舊版每筆一個 spawn，這正是稽核列長年沒有 request_id 的原因。舊資料為 NULL（partial index）。
- SQL 全在 `repositories/audit_logs.rs`（`insert_batch` / `get_audit_logs`）。**middleware 裡不寫 SQL** —— 2026-08-06 前那是全 repo 唯一的破例。
- ⚠️ **登入端點不在稽核範圍內**：middleware 靠 `AuthenticatedUser` extension 認人，而登入時還沒有那個身分（`POST /admin/auth`、`/admin/auth/passkeys/login/finish` 都是公開路由）。所以「誰從哪個 IP 登入成功/失敗」由 `routes/auth.rs` 自己記 log：失敗 WARN、成功 INFO（平時不落地，`log_db_level=INFO` 才收）。**密碼不進 log，帳號名進** —— 那是查「被試的是哪個帳號」的必要資訊。2026-08-08 前這件事兩邊都沒有紀錄。

## 對外連線：connect 階段的偶發失敗（2026-08-09 / 08-10 / 08-11）

`Cannot assign requested address (os error 99)` 中過 OAuth 登入、寄信、docker daemon 自己
`docker pull`。**三個不同的成因，別混為一談**：

1. **宿主機 IPv6 半殘** —— 沒有可用的 IPv6 卻開著 stack，glibc 與 Go 都先試 AAAA 再撞牆。
   已於 08-09 08:46 (UTC) 在宿主機關閉。修好的是 dockerd 與 lettre。
2. **Docker 內嵌 DNS 冷快取回 AAAA-only** —— 容器重啟後第一個對外請求偶爾中；
   hyper 只在解析結果同時有 A 與 AAAA 時才會退回 v4，AAAA-only 就沒有 fallback 可退。
   **第 1 點修完之後這個還在**（08-09 16:14 又中一次）。
3. **解析整個失敗**（08-11 新增）—— errno 是 `EAI_NONAME`
   （`dns error <- failed to lookup address information: Name or service not known`），
   **不是**第 2 點那個「拿到 AAAA 接不上」，而是 A/AAAA 一筆都沒拿到。**與容器重啟無關**：
   發生時 backend 已連續跑 27 小時（最後一次 backend-ci 是 08-09 16:40 UTC）。
   近兩週只中 `www.twse.com.tw`，08-03 / 08-05 / 08-10 / 08-11 各一次、全是單發。
   成因未定（上游 DNS 或內嵌 DNS 轉發抖動皆有可能），但形狀與第 2 點一致：重新 resolve 就通。

**完整證據與推導見 `deploy/README.md`** 的「主機 IPv6」與「對外連線偶發 connect 失敗」兩節。

後端這側：

- `utils/reqwest.rs::send_retrying` —— 對 `is_connect()` / `is_timeout()` 重試 **4 次（250/500/1000ms，合計 1.75 秒）**。
  2026-08-12 從 3 次 / 600ms 放寬：抖動撐過 600ms 就整批失敗、掉到 job 層退避 1800／3600 秒
  （08-11 23:00 的 `fetch_gov_tenders` 就是這樣退避半小時）。
  ⚠️ **呼叫端不只排程 job**（08-13 修掉的錯誤註解）：`services/oauth.rs` 有 5 支在 member
  登入路徑上，抖動時退避會疊加（Google 序列 2 支 → 最壞多 3.5 秒、GitHub 3 支 → 5.25 秒），
  仍在 60 秒 request timeout 內。要再放寬預算前先確認登入路徑吃得下。
  同日另一半修法是 `deploy/docker-compose.yml` backend 的 `dns_opt: [single-request-reopen,
  timeout:2, attempts:3]` —— 三種 errno 的共同根因是內嵌 DNS 把一次解析拆成並行的 A/AAAA 兩問，
  序列化那兩問即可降低發生率；**序列化只在單一 getaddrinfo 內部，不影響併發與承載量**。
  統計與推導見 `deploy/README.md`「三種失敗的共同根因與修法」。
  **只重試「請求還沒送達對方」的失敗**：對方已回狀態碼的不重試，那不是抖動。
  對成因 2、3 這是**對症的解**（重試會重新 resolve），不是將就。
  - 呼叫端：`services/oauth.rs`（token 交換 / Google userinfo / GitHub user+emails / LINE profile）
    ＋ **同檔的 `get_raw_html_string` / `get_json_data`**（2026-08-11 起）—— 後者一補，
    走這兩支的所有排程抓取（TWSE 全部）就一起涵蓋了。
    `services/gov_tenders.rs` 沒走那兩支，同日直接改呼叫 `send_retrying`。
  - **新的對外呼叫一律走 `get_raw_html_string` / `get_json_data` 或 `send_retrying`，不要自己 `.send()`。**
    2026-08-11 起 `src/**` 只剩 `send_retrying` 內部有裸 `.send()`；2026-09-24 起 `clippy.toml` 禁用 `reqwest::RequestBuilder::send`，`send_retrying` 以函式層級 `#[allow]` 豁免。
  - 為什麼要補：TWSE 那條路徑上，一次單發的 DNS 失敗代價不對稱 —— `fetch_stock_day_all`
    的重試在 job 層（`run_with_retries`，退避 **3600 秒**），重新 resolve 其實只要 250ms。
- `services/email.rs` —— 同樣的形狀，但判斷條件是往 source chain 找 `io::Error` 的 kind
  （`AddrNotAvailable` / `ConnectionRefused` / `NetworkUnreachable` / `HostUnreachable` / `TimedOut`）。
  **信一旦進了 SMTP 對話就不重試** —— 重試會讓同一封中獎通知寄兩次，那比慢一輪更糟。
- 兩者都會在重試前記一筆 WARN，所以「有抖動但撐過去了」在 `logs` 表看得到，不會變成靜默。
  ⚠️ reqwest 那筆走 `error_chain()` 印**整條 source chain** —— `reqwest::Error` 的 Display
  只有 `error sending request for url (…)`，errno 全在底下。08-09 追這個問題時就是卡在這裡：
  唯一撈得到 errno 的是 `errors.rs` 用 `{:?}` 寫進 `fields.self` 的那筆 ERROR，
  而**重試成功的請求根本不產生 ERROR**。

⚠️ **寄信失敗一律不標記已通知**（`services/email.rs::SendError` + 三個呼叫端：`jobs/fetch_gov_tenders.rs`、`jobs/fetch_buyback_periods.rs`、`services/torrents/lifecycle.rs`）。
2026-08-05 就是「信沒寄出去、資料照樣 `mark_notified`」讓一筆標案公告永久遺失的 ——
目前只有標案那支真的有 `mark_notified`，但加新的通知路徑時，`mark_notified` 只能放在 `Ok` 之後。

## 系統指標（system metrics）

`jobs/collect_system_metrics.rs` 每分鐘一筆寫 `system_metrics`（保留 90 天，`jobs/cleanup_observability.rs` 清理）；`GET /metrics?hours=`（需 `metric:read`，hours clamp 1–168）。

- **回傳前會依範圍聚合**：`repositories/system_metrics.rs::bucket_seconds`（純函式附測試）算出桶寬，SQL 用 `date_bin` 分桶，目標 ~720 點（圖寬 720px，再細看不出來）。12 小時內桶寬退化成 60 秒＝原始資料；24h→2 分、72h→6 分、168h→14 分。沒有這層 168 小時要吐 10080 筆（≈2MB JSON + 上萬個 SVG 節點）。
- **桶內取 `max` 不是 `avg`**：這頁是拿來找尖峰的，平均會把短暫爆衝抹平。代價是回傳值語意變成「該桶峰值」，前端卡片/圖表文案有標示。原始採樣本身已是整分鐘平均，所以 max = 該桶內最忙的那一分鐘。
- ⚠️ **`get_recent` 的 `GROUP BY` / `ORDER BY` 用的是輸出欄位序號**（`date_bin` 那欄的位置，目前是 12）。別名 `created_at` 與來源欄位同名，PG 遇到歧義會選輸入欄位＝等於沒分桶，所以只能用序號 —— **SELECT 加欄位就要同步改那兩個數字**。

### CPU 採樣：跨間隔相減，且 steal 分開算（2026-08-08 重做）

`cpu_pct` = 本輪與**上一輪** `/proc/stat` 累計值相減，即整個採樣間隔（1 分鐘）的平均。基準存在 `AppState.cpu_times`（`cpu_times()` / `set_cpu_times()`，唯一呼叫端是 `CollectSystemMetrics`），行程剛起來的第一輪沒有基準，整筆跳過不落地（記 debug）。

- ⚠️ **不要改回「當場 sleep 500ms 取兩次」**（2026-08-08 之前的寫法）：那個窗口的起點固定落在 cron 秒 0，而 `structs/jobs.rs` 裡同樣是 `0 * * * * *` 的還有 `FetchHistoricalClosingPrices` 與 `ConsumePendingStockChange` —— 於是每分鐘都系統性只量到那三個 job 同時打 DB 的最忙 500ms。實測 `vmstat` 開機以來平均 busy 3%、`load1` 中位數 0.00（720 筆有 411 筆是 0.00），這頁卻長期顯示 34%，**差 10 倍**。跨間隔相減沒有窗口可挑，也不必 sleep。
- **`steal` 不算進 `cpu_pct`**，另存 `cpu_steal_pct` 欄。混在一起的話「機器真的在忙」與「vCPU 被鄰居搶走」長得一模一樣（這正是查上面那個 34% 時第一輪判斷失準的原因）。判讀：`cpu_pct` 高 → 自己在忙；`cpu_steal_pct` 高 → 找 VPS 供應商。`load` 只反映可執行的 process，**steal 與 irq/softirq 都不進 load**，所以「CPU% 高但 load ≈ 0」永遠要先懷疑量測而不是負載。
- `guest` / `guest_nice` 刻意不加進 total：kernel 已把它們併入 `user` / `nice`，整行加總會重複計算讓分母虛胖。這裡恆為 0（本機是 guest 不是 host），但別留著等踩。
- 純函式 `parse_cpu_times` / `cpu_delta` 附 5 個測試（steal 不算 busy、guest 不虛胖分母、舊 kernel 無 steal 欄、間隔平均、計數器倒退回 None）。
- **`created_at` 是桶的起點**，回應不另外帶桶寬；前端由相鄰點的最小間隔推回桶寬（`metrics-view.tsx` 的 `bucketMs`），用來把選區結束時間補滿最後一個桶。改桶寬邏輯要留意這個隱性契約。

## 單字闖關(vocab)

member 端單字生存模式,**多語言**(en 英文 / ja 日文,`words.language` 區分;`/member/vocab`:`POST /runs` 開局(body 帶 `language`,缺省 en)、`POST /runs/{id}/answer` 答題、`GET /me?language=` 統計)。進行中對局存 Redis(`vocab:run:{id}`,TTL 30 分,**正解只在 server 端**),結束落地 `vocab_runs`(含 language)+ 發 `member_vocab_exp`(**經驗分語言**;`members.exp` 已凍結待清理,等級曲線 `100×(n-1)^1.5` 純函式在 `services/vocab/engine.rs` 附測試)。學習進度逐題 upsert `member_word_stats`。

**週期排行榜**:`GET /member/vocab/leaderboard?language=&period=`(weekly/monthly,缺省 en+weekly;訪客可看,登入多回 `me` 名次)。排「週期內 exp 總和」top 20,複習局不計,同分先達成(最後一局較早)在前;台北時間週界(週一 00:00 / 每月 1 日,`services/vocab/engine.rs::period_start` 純函式附測試)。直接 SQL 聚合 `vocab_runs`(索引 `(language, ended_at)`),無 snapshot 表、無結算 job、無 cache;要做歷史榜/週結算獎勵時再加 snapshot 表。

**多語言鐵律:`repositories/vocab/` 所有「依條件篩選」words / vocab_runs 的查詢都必須帶 language,新查詢不得漏**(漏了會混語言——複習池、干擾項、個人最佳全會串)。例外是**主鍵定位**的查詢(`word_by_id` / `count_mastered_among` / `admin_word_language` / `admin_update_word`),language 由 id 決定,不必也不該再帶——但 `count_mastered_among` 的 id 來源必須是已濾過 language 的清單(目前來自 `review_word_ids`),換來源時要重新確認。最容易漏的位置是**fallback 查詢**:`random_word` 的主查詢與回退查詢兩段都要帶。難度窗口依 RunState 的 `diff_min/diff_max`(開局查該語言題庫上下界)clamp,ja 只有 N5+N4 時不會抽不到字。

**日文專屬**:讀音比對走 `services/vocab_ja.rs::normalize_reading`(NFKC → 轉平假名,長音符「ー」保留原樣;`wana_kana` crate),接受羅馬字/平假名/片假名輸入,與 `words.accepted_readings`(多讀音陣列)任一正規化後完全比對。`QuestionDto` 不下發 reading(讀音是潛在正解),答後 `AnswerResponse.reading` 回饋。ja 拼字題 = 意思→讀音(首假名+拍數提示、無例句挖空)。

**題庫擴充不改程式碼**:
- 英文照 SOP `docs/2026-07-08-vocab-word-seeding.md` + `scripts/add_vocab_words.py`
- 日文照 `scripts/import_jmdict_ja.py`(JMdict × JLPT 字表 extract → LLM 翻中文釋義 → assemble 驗證+組 migration);讀音/詞性取 JMdict 權威值,授權 CC BY-SA 4.0 前端已標註。規格見 `docs/2026-07-10-japanese-vocab-mvp-spec.md`

**後台管理**(`/admin/vocab`,權限 `vocab:read`/`vocab:update`):`GET /words`(篩選 language/difficulty/enabled、q 模糊搜尋、`sort=wrong` 錯最多優先,列含全會員答錯統計)、`PUT /words/{id}`(改釋義/讀音/難度/上下架;**表記與語言不可改**,改表記走 seed。日文更新會驗讀音非空並把主讀音補進 accepted_readings)。前端 `/admin/vocab`(nav「內容」群)。單筆修字/下架走後台,批量仍走 migration。

## CORS

允許來源：從 `app_settings.cors_allowed_origins` 讀取（逗號分隔），**重啟後生效**。預設 `https://kawa.homes`。
沒有 `allow_credentials`，`allow_headers` 只有 `AUTHORIZATION` / `CONTENT_TYPE`，`expose_headers` 只有 `x-request-id`。
值的驗證在 `services/app_settings.rs::validate_cors_allowed_origins`（**必須擋 `*`**，理由見設定表）。  
Method：GET、POST、PUT、DELETE、PATCH。Header：Authorization、Content-Type。

## 相關專案

前端：同 monorepo 的 `../frontend`（Next.js），見 `frontend/README.md`

## 待清理 / 已知技術債

2026-07-31 全後端盤點（當時 `routes/` 32 檔、`services/` 27、`repositories/` 28、13 job、7 遊戲、29 migration）的結果。**2026-09-11 重數：`routes/` 28、`services/` 26、`repositories/` 25、11 job、7 遊戲、36 migration** —— 該日移除記帳／發票／樂透三個功能（2026-09-03 曾記 `services/` 31、migration 35、`routes/` 32、`repositories/` 28、13 job）。下列都**已確認存在、有具體落點**，不必再重新調查。

**原有項**
- `tools`、`roster` 完全公開（無 JWT）。`roster` 為刻意設計；兩者共用同一組 Redis rate limit（20 req/60s per IP）。
  **排班演算法在 `services/roster.rs`**（純函式、13 測）：組一份長度等於人數的環狀 pattern，第 i 位第 d 天讀 `pattern[(i + d) % 人數]` —— 每天所有人讀到的位置正好是 pattern 的一個排列，所以**每日各班人數恆等於 pattern 裡的張數**，覆蓋洞構造上不可能發生。pattern 內「早班在前、晚班在後、段間插休假（Bresenham 平均散開）」負責兩條硬約束（晚班不接隔日早班、連續上班上限）；段組成全同會產生短週期（= 有人班表逐日相同），偵測到就把首尾段互換一個早／晚打散。人力不足時**覆蓋優先**，另回 `RosterWarning`（`understaffed` / `shift_uncovered` / `night_to_morning` / `max_consecutive_exceeded`）。**警告碼刻意不帶中文文案**（後端訊息全是寫死繁中，前端 i18n 才能給 en / zh-CN），字面是前端契約。
- **sqlx 全走 runtime query**（`sqlx::query(` / `query_scalar(` / `query_as::<>`，零 `query!` 巨集）＝**SQL 沒有編譯期驗證**，改欄位名或型別要到 runtime 才炸。取捨是不必在 build 時連 DB；要補的話走 `cargo sqlx prepare` 的 offline 模式（CI 已經有 DB service 的基礎設施）。
- ~~無 `[profile.release]`~~ **已實測，決定維持現況不加**（2026-08-19，本機 4 核 WSL，各單次冷編譯 540 個 dep crate）：baseline 276s / 32.55 MiB / 增量 53s；`lto="thin"` 454s / **33.17 MiB（體積反而 +1.9%**，inline 複製壓過死碼消除）/ 增量 99s；`lto="thin"` + `codegen-units=1` 358s / 28.16 MiB(-13.5%) / **增量 142s(+168%)**。體積收益幾乎全來自 `codegen-units=1`，但每次 push 都付重編代價，而後端瓶頸在 I/O（等 PG / Redis / 外部 HTTP），CPU 收益用不到。執行期效能未量測。**要再提這件事請直接引這組數字，不要重跑**（一輪約 20 分鐘）。⚠️ `panic = "abort"` **永久不能加** —— `routes.rs` 用 `CatchPanicLayer`，abort 會讓 handler panic 殺掉整個 process 而不是回 500。Dockerfile 已有 `strip -s`，故 `strip = true` 對產出無差異。
- ~~`sqlx` 開了用不到的 `any` feature~~（2026-08-19 commit `a9353e7` 已砍）。
- **`tokio-cron-scheduler` 可換成自寫排程，省 10 個 crate**（獨占 `tokio-cron-scheduler` / `croner` / `chrono-tz` / `derive_builder`×3 / `num-derive` / `phf`×2 / `siphasher`）。可行的原因：11 個 job 的 cron 秒數**全部是 0**，形狀只有三種（每分鐘 / 每小時 `:mm` / 每日 UTC `hh:mm`），所以 `cron_expression()` 可換成 enum + 「睡到下一個整分再問每個 job 要不要跑」的迴圈，next-fire 計算變成可單測的純函式（現在 cron 正確性完全外包給 crate、零測試）。**沒做的理由**：動的是全部 11 個 job 的觸發機制，而失敗形式是「某個 job 靜默不跑」，單元測試證明不了真的準時觸發（要本機拿假 schedule 實跑 + 上線觀察一天）。搬過去時這三件現有語意不能掉：`tokio::spawn(...).await` 接 panic（自寫迴圈更需要，panic 會殺掉那條 loop task 讓 job 永久消失）、feature 開關**每次觸發時**檢查（熱更新即時生效）、防重疊跳過時記 WARN。

**可抽共用**
- ~~5 個逐字同形的 `XxxPaginatedResponse`~~（2026-08-03 全數收斂進 `structs/pagination.rs` 的 `Paginated<T>`，`BlogsResponse` 一併降形狀）；~~`StatusFilter` 定義兩次~~（2026-08-09 收進 `structs/pagination.rs`）；~~`NotifyPrefRequest`/`Response` 各一份~~（曾收進 `structs/notify.rs`；該檔已隨 2026-09-11 移除記帳／發票／樂透一併刪除）。
- ~~count/list 的 WHERE 條件逐字重寫兩份~~（2026-08-03 全數抽成共用 const）。**原本記的「8 對」是把所有 count+list 配對都算進去，實際有條件重複的只有 5 處**：`gov_tenders`（`LIST_FILTER`，3 個含 ILIKE 的條件）、`torrents`（`LIST_FILTER`，同一函式內兩查詢）、`blogs` 公開列表（`PUBLIC_FILTER`）、`blogs` 後台列表（`OWNER_FILTER`）、`stocks/changes`（QueryBuilder 改成迴圈套用兩個 builder）。另 3 對的 count 根本沒 WHERE（`messages`、`blog_comments::count_all`）或只有單一等值條件（`blog_comments::count_by_blog`，抽 const 反而更難讀，**刻意留著**）。抽出時已逐一比對兩邊條件，**確認原本全部等價、沒有既存的 total/data 不一致**。
- `games/common/room.rs` 缺 `broadcast_to_room(room, msg, outbox)` → `for &p in &room.players` 手寫 **7 次**（avalon 6 + farm 1，2026-08-26 重數）；N 人房那邊 `err1` 反而是**定義了三份**（`common/room.rs` 的泛型 pub 版 + avalon / farm 各一份私有同名函式），而 2 人框架 `common/service.rs` 沒有這支，inline 的 `flush(state, vec![(who, msg::<E>("error", …))])` 重複 **11 處**；avalon/farm 的 `handle_disconnect` 是純轉發廢函式、`start_game` 骨架逐行對應（合計約 150–170 行可抽成「N 人房」層）。~~`flush` 逐字重複兩份~~（2026-08-06 兩邊都改成委派 `AppState::send_outbox`，剩一行）。
- ~~Redis `user:login:{}` 的 format 字串散在 4 個檔案~~（2026-08-06 收斂：`repositories/redis.rs` 的 `login_key` 私有 + `set_user_login` / `user_login_exists` / `del_user_login` 三件套，format 字串只剩一處；附 2 個真實 Redis 測試守著 set→exists→del 與 TTL 存在）。
- ~~`repositories/redis.rs` 5 個 TTL 全是裸數字~~（2026-08-06 全部命名：`LOGIN_TTL_SECS` / `IDENTITY_TTL_SECS`（當時叫 `PERMISSIONS_TTL_SECS`，2026-08-09 併成身分快取時改名）/ `OAUTH_STATE_TTL_SECS` / `MEMBER_REFRESH_TTL_SECS` / `WS_TICKET_TTL_SECS`；名字沒暗示會設 TTL 的 `redis_set()` / `redis_check_key_exists()` 一併刪掉）。
- `services/portfolio/pricing.rs` 兩個 ~80 行的快取管線（Redis→DB→上游→寫回）骨架同構；~~`normalize_optional` 兩份~~（2026-08-09 收進 `utils/text.rs`，附 3 測）；~~`unprocessable` 3 份~~（收進 `errors.rs`，簽名統一成 `impl Into<String>`）；~~權限載入的 Redis→DB fallback 兩份~~（收成 `services::auth::load_identity`，見「身分快取」）。
- `services/stocks.rs` 手刻民國年解析，而**同檔已經 import 了 `utils::date::parse_roc_date`** 並在別處用過。反方向（西元→民國）在 `jobs/fetch_buyback_periods.rs` 一份 —— `utils/date` 只有 parse 沒有 format，該補 `to_roc_compact()`（`1911` 這個常數目前散在 3 個檔案：`services/stocks.rs`、`jobs/fetch_buyback_periods.rs`、`utils/date.rs`）。
- ~~「今天」有三套定義~~（2026-08-06 收斂進 `utils/date.rs` 的 `taipei_offset` / `taipei_now` / `taipei_today`，附 2 測）。**新程式碼一律用這三個，不要用 `Local::now()`**（2026-09-24 起 `clippy.toml` 的 `disallowed-methods` 禁用，CI 紅燈）：`Local` 的結果取決於行程的 `TZ`。生產 image **現在**有設（`Dockerfile` 的 `ENV TZ=Asia/Taipei`，2026-07 併 monorepo 時加的），但那是部署層的規則，拿掉或換掉沒有 tzdata 的基底就悄悄退回 UTC，台北 00:00–08:00 那八小時的「今天」全變昨天，且無任何徵兆。導入這幾支時 image 確實沒設 `TZ`，`services/portfolio.rs` ×3、`services/stocks.rs`、`jobs/fetch_buyback_periods.rs` 都中過。`repositories/visitors.rs::taipei_today` 已移除，改由 utils 提供。
- `services/gov_tenders.rs` 仍繞過 `utils/reqwest` 的 `get_raw_html_string` / `get_json_data`，**沒有狀態碼檢查**（後果較輕：會噴 Json error 而非靜默回空）。2026-08-11 已改呼叫 `send_retrying`，所以缺的只剩狀態碼檢查那半。
- `structs/images.rs` 的 `ImageRecord` 沒 derive `sqlx::FromRow`，於是 `repositories/images.rs` 手動 `row.get(...)` 映射貼 4 次 —— 全 repo 只有這支這樣寫。（型別本身已在 2026-08-09 搬進 `structs/`，缺的只剩 derive。）

**分層**
- ~~`repositories/{logs,audit_logs,system_metrics,visitors}.rs` 回 `sqlx::Error` / `redis::RedisError`~~（2026-09-24 全改 `AppError`，連同 `repositories/redis.rs` 的 `set_user_login` / `user_login_exists`；`services/stats.rs` 的三層 `map_err` 與 `services/logs.rs`、`services/system_metrics.rs::recent` 的轉型殼一併拆掉）。**刻意留著的兩支**：`redis.rs::get_redis_conn`（連線原語，把 bb8 的 `RunError` 攤平成 `RedisError`，由呼叫端 `?` 轉型）與 `visitors.rs` 私有的 `record_visit_inner`（best-effort，只在本檔 warn）。⚠️ 換成 `AppError` 之後**錯誤要用 `{:?}` 印**：它的 Display 只有「系統錯誤: 資料庫錯誤」，sqlx / redis 的原因在 `#[source]` 裡（`jobs/collect_system_metrics.rs` 的 `{e}` 就是因此改成 `{e:?}`）。
- ~~5 個帶 `Serialize` 的 API 回應型別長在 `repositories/`~~（2026-08-09 全數搬進 `structs/`，`routes/admin_stats.rs` 的 `VisitorsStats` 一併搬到 `structs/stats.rs`）。**仍待搬**：~~`routes/ws.rs` 的 `pub SendMessageParams`~~（2026-09-24 已搬 `structs/ws.rs` 並改名 `SendMessageRequest`，wire 形狀不變）、`routes/auth.rs` 的 `MeResponse`、`routes/oauth.rs` 的 `OAuthUrlResponse`。
- ~~`repositories/members.rs` 的 `member_detail` 打 3 次 DB~~（2026-08-09 收成 2 支併發：`members` 一次取齊、`member_oauth` 不依賴前者）。
- ~~4 個端點有分頁但完全沒有 total~~（2026-08-07 補齊，含 `/admin/audit_logs`、`/members` 與 `/admin/stocks/day_all`）。**吃 `page`/`per_page` 的端點一律回 `{data,total}` 這個形狀**，沒有例外（型別上唯一還沒走 `Paginated<T>` 的是 `/admin/vocab/words` 的 `structs/vocab.rs::AdminWordListResponse` —— 逐字同形，是 2026-08-03 那波收斂之後才長出來的第 6 份，wire 形狀相同故不影響前端）。理由不是對稱：前端 `usePagedList` 是全站唯一的「載入更多」實作，缺 total 時它只能猜「這頁滿了就假設還有下一頁」，於是**所有**清單（含有 total 的那些）都退化用同一套啟發式，最後一頁剛好滿 per_page 就多出一顆按不出東西的按鈕。COUNT 的成本用 `tokio::try_join!` 與 list 併發吸收（範本 `services/logs.rs`），list 與 count 的 WHERE 一律抽成同一個 `XXX_FILTER` 常數，兩邊漂移會讓 total 對不上。
- ~~`gov_tenders` / `messages` / `blog_comments` 的 count/list 用順序 `.await`~~（2026-08-09 全改 `tokio::try_join!`；`services/stats.rs` 的三支查詢同時收斂）。
- ⚠️ **`services/portfolio/math.rs` 的 `compute_latest` / `build_history` 在有股票股利時金額算錯**（尚未修，需先確認語意）：除權（`stock_rate > 0`）時只把 `adjusted_cost` 往下調，`shares` 沒跟著放大 —— 真實部位股數是 `shares × (1 + stock_rate/1000)`，所以 `current_value` 與 `pnl` 兩個**金額**欄位都被低估同一個倍數，而 `pnl_pct` 因為分子分母約掉了反而是對的。「pct 對、金額錯」這個內部矛盾在任何語意下都成立。要決定的是 `shares` 是否預期由使用者自己改。**2026-09-08 更新**：還原因子已收斂成單一 `ex_adjust_factor`（三個呼叫端共用），所以只要改一處；`period_change` 的 `value_change` 也吃同一個 `shares`、同樣被低估。該檔已有 6 測，但全在 `changes` 那組，沒有一個覆蓋這個 bug。
- ~~`services/email.rs` 每封信重建 `AsyncSmtpTransport`~~（2026-08-09：開 lettre `pool` feature + `static MAILER` 依憑證快取 transport，憑證變了才重建。**兩者缺一都沒用** —— 沒開 pool 就算重用 transport 也是每封一次握手，重建 transport 則等於重建連線池）。
- `routes/oauth.rs` 是 28 個模組裡唯一在自己內部提前 `.with_state(state)` 的（其他都回無 state 的 `Router<AppState>` 由 `routes.rs` 統一注入）。

**拆檔**（比照 `repositories/stocks/` 這個好範本）—— ~~2026-09-24 全數完成~~，做法一致：`<name>.rs` 改成 `<name>/mod.rs` + 子模組 + `pub use 子模組::*`，**呼叫端路徑完全不變**、函式內容逐字搬移；子模組之間互用的私有 item 開成 `pub(super)`（只對同一個父模組可見），從定義處直接 `use super::<子模組>::x`。
- `services/vocab/`（原 982 行）：`engine`（全部純函式 + 全部測試）/ `question` / `run` / `answer` / `stats` / `admin`
- `services/torrents/`（原 960）：`manager` / `tasks`（對外 CRUD + 擁有者檢查 + 排程清理）/ `lifecycle` / `session` / `download_links` / `storage`
- `repositories/vocab/`（原 527）：`words` / `stats` / `runs` / `admin`
- `services/portfolio/`（原 755）：`crud` / `pricing`（快取管線 + 上游預算）/ `math`（純計算 + 測試）
- `routes/ws.rs`（原 501 → 167 行）：socket 生命週期搬到 `services/ws/socket.rs`，後台連線清單與點對點送訊的邏輯抽成 `services/ws/connections.rs`，原 `services/ws.rs` 拆成 `ticket` / `guard`。route 只留握手（身分、per-IP 上限、到訪採集、span）與三支 admin handler 的權限檢查。⚠️ **WS 的 log target 因此從 `api_server_axum::routes::ws` 變成 `api_server_axum::services::ws::socket`**（repo 內無人依賴舊值；用 `kawa-logs` / 後台 `target` 篩選時注意）。2026-09-24 本機端到端實測 9 項（匿名遊戲分派、ticket 換 admin 連線、連線清單排序與時間格式、點對點送訊 200 / 400 / 404、`user_left`、灌 80 則收到 `rate_limited` 且不斷線、連線結束摘要）全過。

**測試空缺**
- ~~`services/portfolio.rs`（557 行，**0 測**）、還原公式逐字複製兩份~~（2026-09-08 隨期間增減一併處理：公式收斂成單一 `ex_adjust_factor`（`compute_latest` / `build_history` / `period_change` 共用），並補 6 個純函式測試 —— 全在 `changes` 那組（前一交易日基準、單日無增減、除息日不算大跌、週/月取不晚於目標日的最近交易日、太新的持股回 `None`、基準過舊寧可回 `None`））。**仍缺的是 `build_history` 自己的測試**（逐日還原成本那條路徑目前零覆蓋），以及下面那條金額算錯的 bug。
- `games/common/room.rs`(380) + `common/service.rs`(520) —— 7 個遊戲共用的 900 行框架 **0 測試**，而本檔逐一列出 7 個 engine 的 76 個測試。框架已用 `outbox: Vec<(SocketAddr,String)>` 把 IO 隔離掉，測試不需要真 WS。**一個 bug 在這裡 = 7 個遊戲一起壞。**
- ~~`structs/pagination.rs::to_limit_offset`~~（2026-08-03 補了 4 個 clamp / page 邊界測試）；`structs/roles.rs` 的 `Perm::as_str`（缺 `Feature` 那種 round-trip 測試，permission string 打錯字 = **靜默 403**）；`services/twse.rs`；`services/vocab/engine.rs` 的 `pick_kind` 與 `resolve_duration_minutes`（同檔其他 8 個純函式有測 —— `exp_for_level` / `level_for_exp` / `answer_exp` / `difficulty_window` / `clamped_window` / `mask_sentence` / `period_start` / `streak_from_days`，這兩個漏）。

**其他觀察**
- `games/common/room.rs` 的 `start_check` 用 `h.rooms.get(&room_id).unwrap()`，而同檔 `remove_from_room` 對同一組查詢用 `let Some(...) else { return }` 防禦 —— 目前 invariant 成立故不可達，但兩種寫法並存。
- `farm/hub.rs` 設 `SEAT_IN_ROOM_UPDATE = true`、avalon 吃預設 `false`，但註解說的成因（`common/room.rs` 的座位重編號）**兩個遊戲都有**。實際理由是 farm 要判 `myTurn`（`current_player == mySeat`）且沒有私有訊息可夾帶，avalon 的 seat 從私有 `role_assigned` 拿 —— ~~註解該改成這個理由~~（2026-09-07 commit `235d764` 已改，`farm/hub.rs` 現在寫的就是這個理由，並註明重編號解釋不了差異）。（前端 avalon 的 `iAmHost` 用本地 flag 是**正確的**，因為 host 離開＝解散房間，重整後沒有房可以回。）
- `services/vocab/answer.rs` 的 `member_id.expect("review mode requires member")` —— `run` 是從 Redis 反序列化來的，若曾寫入過 `mode=review` 且 `member_id=None` 的狀態就會 panic，該回 `AuthError::Unauthorized`。
- `state.rs` 三次 `write()` 不是原子的，設定更新瞬間可能讀到混合狀態（影響輕微，設定更新極罕見）。~~`AppStateInner` 13 個欄位宣告 `pub`~~（2026-09-24 struct 與欄位全改私有；`AppState` 的 tuple 欄位本來就私有，所以封裝其實一直由編譯器守著，這次只是拿掉誤導的 `pub`）。
- ~~job panic 不致命但**無痕**~~（已修：`scheduler.rs` 把 `job.run` 包在自己的 `tokio::spawn` 裡再 `await`，`e.is_panic()` 時把 payload 取出記 `error!` —— tokio-cron-scheduler 自己不 join handle，不包這層的話 panic 就只剩 runtime 預設 hook 印的那行 stderr，不進 tracing，而生產 image 無 shell。防重疊的 `running` guard 是 `MutexGuard`，unwind 時正常 drop，**不會卡死後續 tick**。）
- `migrations/20260713000000_logs_retention.up.sql` 的註解寫「清理 job 見 `jobs/cleanup_logs.rs`」，該檔早已改名 `cleanup_observability.rs`。**不能改** —— sqlx 會驗已套用 migration 的 checksum，改了下次啟動直接失敗。已套用的 migration 內容（含註解）一律不可動。
- `20260728000000_media_domain_backfill` 的 down 是**刻意 no-op**（註解說明反向替換會連新圖一起改壞），是 36 個 migration 中唯一一個 —— `sqlx migrate revert` 跑它會「成功」但什麼都沒回退。**此 migration 不可 revert，要回退請從備份還原。**
- migration 命名有兩套：34 個是手寫整點時間戳（`HHMMSS` = `000000`/`100000`…），但 `20260710155453_vocab_seed_ja_pilot` 與 `20260710162925_vocab_seed_ja_n5n4` 是 `sqlx migrate add` 生成的真實秒級時間戳。功能無害（排序仍正確），但下一位開發者不知道該用哪種。

**已查證「不是問題」，別再動**
- `AppStateInner` 封裝零違反；sync `RwLock` 跨 `.await` 零違反（三個 getter 都在同一 expression 內取完釋放，`get_settings()` 只 clone Arc）；Redis TTL 遺漏零筆（`cache_set` 把 `ttl_secs` 設成必填參數，型別層就不可能忘）；handler 內零 inline SQL；11 個 job 的四層註冊全對；7 個遊戲的 engine 測試數與本檔聲稱**完全相符**（2026-08-17 複驗，2026-09-11 重數仍是 76）；36 個 migration up/down 100% 成對；vocab 多語言 language 零遺漏。
- **`#[serde(flatten)] PageQuery` 行不通**，別再嘗試（詳見 `structs/pagination.rs` 的註解）。
- **直接依賴沒有「完全沒用到」的了**（2026-08-19 逐一掃過 41 個）：當時只揪出 `tokio-util`（src 零引用）、`futures-util`（與 `futures` 重複）、`reqwest` 的 `multipart` feature（multipart 只發生在 axum 收上傳那側），已於 commit `f205d7f` 砍掉；`scraper` 改 regex（`83d66fa`）。其餘全部有實際 `use`。⚠️ **`openssl` 是唯一「src 零引用但必留」的** —— 它存在只為替 `webauthn-rs-core` 的 transitive openssl 開 `vendored`，砍掉會回頭吃系統 libssl（見上面「依賴約束」）。⚠️ **刪 manifest 條目不等於少編一個 crate**：`tokio-util` / `futures-util` 被 redis / librqbit / axum / bb8 各自拉，那次三項合計 crate 數 **0 減少**（純 manifest 噪音）；真正減量要看「獨占 crate 數」（`scraper` -28、`tokio-cron-scheduler` -10）。
