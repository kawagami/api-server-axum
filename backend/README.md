# Rust Axum API Server

Rust + Axum 網頁 API 伺服器，部署於 `https://api.kawa.homes`（舊名 `axum.kawa.homes` 仍為有效 alias）。

## 功能

- JWT 驗證（admin 登入 / 登出 / token refresh / Redis session）
- OAuth 登入（Google / GitHub / LINE）+ member token refresh
- WebSocket 即時推送（逐連線 sender，廣播直接迭代連線 map）
- 部落格 CRUD + 文章留言
- Admin passkey 登入（WebAuthn，密碼登入的可選升級）
- Runtime 設定管理（admin 頁面熱更新，不需重啟）
- instance 功能開關（`enabled_features`，關閉的功能連路由都回 404）
- 股票資料（全市場行情、庫藏股計畫、股價變動追蹤）
- 圖片上傳 / 管理（本機儲存）
- Torrent 下載（磁力連結 → 內嵌 librqbit session 下載 → 短效簽名連結取檔，併發上限 / 容量配額 / 完成 email 通知）
- 使用者 / 角色 / 權限管理
- 投資組合管理（member 持股 CRUD）
- 記帳（member 收支記錄 CRUD，固定分類，收支結餘 / 分類加總 / 每月趨勢統計）
- 發票登錄 + 統一發票自動對獎（member 登錄發票，排程每期抓財政部中獎號碼比對，中獎寄 email 通知，opt-in）
- 樂透登錄 + 大樂透 / 威力彩自動對獎（member 批次登錄選號，排程每日抓台彩開獎號碼比對，中獎寄 email 通知，opt-in）
- 排班（roster）
- 單字闖關（member 生存模式，英文 / 日文，週期排行榜）
- 政府採購網標案追蹤（依關鍵字每日抓取，新公告 email 通知）
- 站內留言板
- 觀測（應用日誌落地 `logs` 表、每分鐘系統指標 `system_metrics`、操作稽核 `admin_audit_logs`）
- 每日不重複到訪統計（WS 握手採集、Redis HyperLogLog 計數、每日落地 `daily_visitor_stats`）
- 排程 job（cron）
- 線上對戰遊戲（象棋 / 五子棋 / 暗棋 / 西洋棋 / 圍棋 / 阿瓦隆 / 農場經營，server-authoritative，匿名，大廳自選桌 + 快速配對，複用 `/ws`；泛型框架 + Fischer 計時；阿瓦隆＝N 人社交推理、農場經營＝N 人 worker-placement）

## API 路由

| 前綴 | 說明 |
|------|------|
| `/admin/auth` | admin 登入、me、token refresh、passkey 註冊 / 登入 / 管理 |
| `/admin/users` | 使用者管理 |
| `/admin/roles` | 角色管理 |
| `/admin/permissions` | 權限清單 |
| `/admin/settings` | Runtime 設定（讀取 / 更新） |
| `/admin/audit_logs` | 操作稽核紀錄 |
| `/admin/blogs` | 部落格列表（分頁 + `?tag=&q=&sort=`）/ 修改 / 刪除 / tag 改名合併 |
| `/admin/images` | 圖片上傳 / 刪除 / 清單 |
| `/admin/stocks` | 股票資料查詢、pending change 管理 |
| `/admin/torrents` | torrent 下載任務（新增 / 列表 / 簽名下載連結 / 刪除） |
| `/admin/games` | 即時對局總覽（各遊戲等待 / 進行中桌數、在玩人數、排隊、大廳） |
| `/admin/stats` | 每日不重複到訪統計（today 即時 PFCOUNT + 近 N 天去重 + 歷史） |
| `/admin/gov_tenders` | 政府採購網標案列表 / 類型清單（需 `gov_tender:read`） |
| `/admin/vocab` | 單字題庫管理（列表 / 修改單字，需 `vocab:read` / `vocab:update`） |
| `/admin/messages` | 站內留言管理 |
| `/admin/blog_comments` | 文章留言管理 |
| `/oauth` | member OAuth 登入（Google / GitHub / LINE）、token refresh |
| `/members` | member 管理 |
| `/member/portfolio` | member 投資組合 CRUD、即時損益總覽、歷史價格 / 還原成本（需 Bearer token） |
| `/member/ledger` | member 記帳 CRUD、固定分類清單、收支 / 分類 / 每月統計（需 Bearer token） |
| `/member/invoices` | member 發票登錄 CRUD、中獎 email 通知開關（需 Bearer token；對獎由排程處理） |
| `/member/lotto` | member 樂透選號批次登錄、列表 / 開獎結果查詢、中獎 email 通知開關（需 Bearer token；對獎由排程處理） |
| `/member/vocab` | 單字闖關開局 / 答題 / 個人統計 / 週期排行榜（en / ja） |
| `/admin/invoice_lottery_numbers` | 手動補統一發票中獎號碼（需 `invoice_lottery:write`，自動抓取失敗時的後備） |
| `/settings/public` | 公開設定（白名單，如 `site_theme`，無認證） |
| `/blogs` | 部落格查詢（列表 / tags / 單篇，公開） |
| `/messages` | 站內留言 |
| `/ws` | WebSocket 連線、線上清單（`/ws/connections`）、點對點訊息（`/ws/messages`）、一次性連線票（`/ws/ticket`）、對戰遊戲配對/對戰（象棋/五子棋/暗棋/西洋棋/圍棋/阿瓦隆/農場經營） |
| `/roster` | 排班 |
| `/logs` | 應用日誌查詢（列表 + 單一 request 完整軌跡，需 `log:read`） |
| `/metrics` | 系統指標時間序列（需 `metric:read`，`?hours=` clamp 1–168） |
| `/uploads/*` | 本機靜態檔案 |
| `/tools` | 工具 |
| `/health` | 存活探針，回 `200 {"status":"ok"}`（無認證、不查 DB / Redis，給外部 uptime 監控用） |

分頁端點統一 `?page=1&per_page=N`（per_page 上限 200）；POST 建立資源回 `201`，更新／刪除無內容回 `204`。

根路徑 `/` 沒有路由，打了回 `404` —— 這是正常的，要探測服務是否活著請打 `/health`。
所有 404（含未知路徑、功能開關關閉）與其他錯誤共用同一個 JSON 形狀
`{ code, message, details?, request_id }`，`request_id` 同時在 `x-request-id` header,回報問題時可據此撈 log。

## 排程 Job

| Job | 週期 | 說明 |
|-----|------|------|
| `ConsumePendingStockChange` | 每分鐘 | 消費一筆 pending stock_change，查詢 TWSE 股價 |
| `FetchHistoricalClosingPrices` | 每分鐘 | 補缺起始日收盤價 |
| `CleanupUnusedImages` | 每小時 | 清除 status=unused 且逾時的孤立圖片 |
| `CleanupExpiredTorrents` | 每小時 :30 | 清除逾期 torrent（DB + 磁碟） |
| `CollectSystemMetrics` | 每分鐘 | 採一筆系統指標寫入 `system_metrics` |
| `CleanupObservability` | 每日 UTC 16:20 | 清理逾期的 `logs`（14 天）/ `system_metrics`（90 天）/ `admin_audit_logs`（180 天） |
| `FetchStockDayAll` | 每日 UTC 20:00 | 抓全市場行情寫入 `stock_day_all` |
| `FetchBuybackPeriods` | 每日 UTC 20:00 | 抓庫藏股計畫 HTML 寫入 `stock_buyback_periods`；有新未來庫藏股時 email 通知（需設定 `smtp_username` / `smtp_password`） |
| `SyncBuybackToPending` | 每日 UTC 20:10 | 將 `stock_buyback_periods` 同步為 pending stock_changes；若 end_date 有異動，自動更新 pending 狀態的記錄 |
| `CheckInvoiceLottery` | 每日 UTC 17:00 | 抓財政部統一發票中獎號碼，對 member 登錄發票比對，中獎且已開啟通知者寄 email |
| `CheckLottoWins` | 每日 UTC 13:30 | 抓台彩大樂透 / 威力彩開獎號碼，對 member 登錄選號比對，中獎且已開啟通知者寄 email |
| `AggregateVisitors` | 每日 UTC 16:05（台北 00:05） | 落地前一台北日不重複到訪 PFCOUNT → `daily_visitor_stats` |
| `FetchGovTenders` | 每日 UTC 23:00（台北 07:00） | 依 `gov_tender_keywords` 抓政府採購網標案，新公告寄 email 通知 |

共 13 支，權威清單在 `src/structs/jobs.rs` 的 `AppJob::ALL`（`scheduler.rs` 從那裡迭代）。

## 技術棧

- `axum 0.8` — router、multipart、WebSocket
- `sqlx 0.8` — async PostgreSQL + 自動 migration
- `bb8` + `bb8-redis` — Redis 連線池
- `tokio-cron-scheduler` — cron job
- `jsonwebtoken 9` — JWT
- `reqwest 0.12` — 對外 HTTP 請求
- `tower-http` — CORS、timeout、trace、body limit（10 MB）
- `tower` — ServiceExt::oneshot（檔案下載走 ServeFile，內建 Range）
- `bcrypt` — 密碼 hash
- `lettre` — SMTP email 通知
- `regex` — 民國日期 / 中獎號碼 feed / 庫藏股 HTML 解析（後者原用 `scraper`，2026-08-19 改 regex，省 28 個 crate）
- `librqbit 8` — 內嵌 BitTorrent session（rustls）
- `image 0.25` + `webp 0.3` — 圖片上傳 decode 驗證 + lossy WebP 轉檔（libwebp）
- `webauthn-rs 0.5`（+ `webauthn-rs-proto`）— admin passkey 登入；硬依賴 OpenSSL ≥3.0，故 openssl crate 開 `vendored`
- `wana_kana 5` — 日文讀音正規化（羅馬字 / 平假名 / 片假名互轉）
- `zhconv 0.4` — 繁簡轉換（`/tools/convert_text`）

## 環境變數

| 變數 | 必填 | 預設值 |
|------|------|--------|
| `DATABASE_URL` | 是 | — |
| `REDIS_URL` | 是 | —（完整 URL，如 `redis://valkey:6379`；可帶密碼 / `rediss://` / db index） |
| `JWT_SECRET` | 是 | — |
| `APP_HOST` | 否 | `0.0.0.0` |
| `APP_PORT` | 否 | `3000`（**僅限本機直跑**；生產的 3000 被 nginx upstream 與 `API_URL` 寫死，改這個只會 502，故 `kawa.env` 不放這個 key） |
| `UPLOAD_PATH` | 否 | `./uploads` |
| `TORRENT_PATH` | 否 | `./torrents` |
| `TRUST_CF_HEADER` | 否 | `false`（true/1 才信任 CF-Connecting-IP，僅限只經 Cloudflare 的部署） |
| `RUST_LOG` | 否 | 未設時 release build 用 `info,tower_http=warn`、debug build 用全 `debug`（`main.rs::default_log_filter`）。這是 stdout 的 filter，也是 `logs` 表門檻（`app_settings.log_db_level`）的天花板 |
| `GOOGLE_CLIENT_SECRET` | 否 | — |
| `GITHUB_CLIENT_SECRET` | 否 | — |
| `LINE_CLIENT_SECRET` | 否 | — |

## 常用指令

```bash
docker build --no-cache -t kawagami77/api-server:axum .   # Docker build
cargo watch -x run                                        # 本機開發熱重載
```

## 在 VPS 環境中給予指定 user 角色(super_admin)的指令
```
docker exec -it database psql -U USER -d DATABASE -c "INSERT INTO user_roles (user_id, role_id) SELECT u.id, r.id FROM users u, roles r WHERE u.email = 'kawa@gmail.com' AND r.name = 'super_admin';"
```
