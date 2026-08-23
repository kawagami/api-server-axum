use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::torrents as torrents_repo,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::Paginated,
        torrents::{
            DownloadLink, Torrent, TorrentDownloadClaims, TorrentFile, DOWNLOAD_TOKEN_PURPOSE,
            STATUS_COMPLETED,
        },
        ws::WsEvent,
    },
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use librqbit::{
    api::TorrentIdOrHash, AddTorrent, AddTorrentOptions, AddTorrentResponse, Magnet, ManagedTorrent,
    Session, SessionOptions,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};

/// metadata 解析的**讓位檢查間隔**預設值（秒）— 可由 app_settings.torrent_metadata_timeout_seconds
/// 熱更新（key 名沿用歷史，語意已不是硬逾時）。到點只檢查有沒有任務排隊等名額：
/// 沒有就繼續等（冷門種子需要時間），有才放棄本輪。
const DEFAULT_METADATA_TIMEOUT_SECONDS: i64 = 180;
/// 讓位次數上限：被排隊任務擠掉這麼多次仍沒拿到 metadata 才判 failed。
/// 沒人排隊時不會累計，所以「一直等」不會用掉額度。
const MAX_METADATA_ATTEMPTS: i32 = 3;
/// 初始化逾時 — metadata 到手後驗證磁碟既有 piece（純本地 IO + SHA-1，大檔在小機器上會久）
const INIT_TIMEOUT: Duration = Duration::from_secs(1800);
/// 進度輪詢間隔
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// 下載連結效期預設值（分鐘）— 可由 app_settings.torrent_link_ttl_minutes 熱更新
const DEFAULT_LINK_TTL_MINUTES: i64 = 180;

const DEFAULT_MAX_ACTIVE: usize = 2;
const DEFAULT_MAX_TOTAL_SIZE_GB: i64 = 20;

/// 啟動失敗的分類 —— 讓位還有重試機會，其他錯誤直接判 failed
enum StartFailure {
    /// 檢查點到了還沒找到 peers，而且有任務排隊等名額 → 讓位（額度未用完只是排到隊尾）
    MetadataTimeout,
    Fatal(String),
}

/// 進行中任務的一格。
/// **佔位早於 `add_torrent`**：magnet 的 metadata 解析在 librqbit 內部進行、可能耗上數分鐘，
/// 那段期間還沒有 handle，但名額已經被吃掉，不先佔位就會被重複啟動、併發數也算不準。
struct Slot {
    /// 整條生命週期的 task（啟動 + 監看），刪除時 abort
    task: JoinHandle<()>,
    /// metadata 解析完才有；None = 還在解析
    handle: Option<Arc<ManagedTorrent>>,
}

/// torrent session 與進行中任務的 handle 對照表。
/// 進度不落 DB — 即時資訊一律從 handle 讀。
pub struct TorrentManager {
    session: Arc<Session>,
    active: Mutex<HashMap<i32, Slot>>,
    base_path: PathBuf,
}

impl TorrentManager {
    pub async fn new() -> Self {
        let base_path =
            PathBuf::from(std::env::var("TORRENT_PATH").unwrap_or_else(|_| "./torrents".to_string()));
        tokio::fs::create_dir_all(&base_path)
            .await
            .expect("failed to create TORRENT_PATH");
        // `ipv4_only: true` —— 不能用 `Session::new`（它等於全默認）。
        //
        // librqbit 9 的招牌改動是「預設全走 dualstack IPv6 + IPv4」：
        // `SessionOptions::default()` 的 `ipv4_only` 是 false、`dht` 是 Some，於是 DHT
        // 的 UDP socket 綁 `[::]`、outgoing peer 連線也會去撥 v6 peer。
        // 而這台 VPS 從來沒有可用的 IPv6（無全域位址、無 `::/0`），宿主機已於
        // 2026-08-09 整台關掉 —— 理由見 deploy/README.md「主機 IPv6：必須整台關掉」。
        //
        // ⚙ 2026-08-23 升 9 時在 production 量過這三格（在 backend 的 netns 內）：
        //   /proc/sys/net/ipv6/conf/all/disable_ipv6 → 1
        //   /proc/net/if_inet6                       → 檔存在但內容全空（模組有載、零位址）
        //   bind(AF_INET6, "::", 0)                  → **成功**
        // 所以這行**不是在防 panic** —— wildcard 綁得起來，Session 建得起來。
        // 它防的是【靜默浪費】：DHT 學到的 v6 節點、tracker 回的 compact6 peer，
        // 每一發都撒進死巷，而 librqbit 的 target 不在 EnvFilter 的白名單上
        // （預設 `{crate}=info,tower_http=warn`，見 main.rs::default_log_filter）
        // → 它的 WARN 進不了 stdout 也進不了 logs 表，**你一行都看不到**。
        //
        // ⚠ `ipv4_only` **不是全覆蓋的**（librqbit 9.0.1 實測，數 /proc/self/fd 對
        // /proc/self/net/udp6 得到的結果，見本檔 tests）:
        //   DHT           → 吃 `ipv4_only`，綁 0.0.0.0 ✅
        //   outgoing peer → 吃 `ipv4_only` ✅
        //   LSD           → **不吃** —— session.rs 建它時只傳 `..Default::default()`，
        //                   `ipv4_only` 沒被轉進去，會多一個 `[::]:6771` 的 v6 多播 socket。
        //                   ⇒ 直接 `disable_local_service_discovery: true`:LSD 找的是
        //                   **同網段**的 peer，VPS 上沒有這種鄰居，本來就是純浪費。
        //   UDP tracker   → **不吃**，而且**改不了** —— tracker_comms_udp.rs 把 `[::]:0`
        //                   寫死。但 `BindOpts::default()` 是 `request_dualstack: true`
        //                   （→ `only_v6=0`），所以它靠 v4-mapped 走 v4 出去，功能正常。
        //                   代價:那顆 socket 是 AF_INET6。**這是脆弱點** —— 哪天 IPv6
        //                   模組真的不在（`ipv6.disable=1`），`socket()` 會回 EAFNOSUPPORT，
        //                   `Session::new*` 直接 Err → 這裡的 `.expect` 就會炸掉整個 backend。
        //                   現況安全的依據是上面量到的「模組有載、wildcard 綁得起來」。
        //
        // 哪天真的把 IPv6 設起來（provider 給 /64 + Docker 網路 enable_ipv6），
        // 這行跟 nginx 的 `listen [::]` 一起拉回來。
        let session = Session::new_with_opts(
            base_path.clone(),
            SessionOptions {
                ipv4_only: true,
                disable_local_service_discovery: true,
                ..Default::default()
            },
        )
        .await
        .expect("failed to create torrent session");
        Self {
            session,
            active: Mutex::new(HashMap::new()),
            base_path,
        }
    }

    pub fn base_path(&self) -> &std::path::Path {
        &self.base_path
    }

    pub fn output_dir(&self, info_hash: &str) -> PathBuf {
        self.base_path.join(info_hash)
    }

    pub async fn get_handle(&self, id: i32) -> Option<Arc<ManagedTorrent>> {
        self.active.lock().await.get(&id).and_then(|s| s.handle.clone())
    }
}

/// 讀 app_settings 的數值設定，缺失/壞值一律回退預設（型別由 default 決定）
fn setting<T: std::str::FromStr>(state: &AppState, key: &str, default: T) -> T {
    state
        .get_settings()
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

/// 併發上限（正在解析 metadata 的佔位也算一格）
fn max_active(state: &AppState) -> usize {
    setting(state, "torrent_max_active", DEFAULT_MAX_ACTIVE)
}

/// 解析 magnet URI，回傳小寫 hex info_hash
pub fn parse_info_hash(magnet_uri: &str) -> Result<String, AppError> {
    let magnet = Magnet::parse(magnet_uri)
        .map_err(|e| RequestError::UnprocessableContent(format!("無效的磁力連結: {e}")))?;
    let id20 = magnet
        .as_id20()
        .ok_or_else(|| RequestError::UnprocessableContent("磁力連結缺少 btih info hash".to_string()))?;
    Ok(id20.as_string())
}

/// 新增任務：容量檢查 → 寫入 pending → 嘗試啟動
pub async fn create(state: &AppState, magnet_uri: &str, created_by: &str, owner_id: Option<i64>) -> Result<Torrent, AppError> {
    let info_hash = parse_info_hash(magnet_uri)?;

    let max_bytes = setting(state, "torrent_max_total_size_gb", DEFAULT_MAX_TOTAL_SIZE_GB)
        .saturating_mul(1024 * 1024 * 1024);
    let used = torrents_repo::total_size_sum(state.get_pool()).await?;
    if used >= max_bytes {
        return Err(RequestError::InsufficientStorage(format!(
            "torrent 已用容量 {used} bytes 達上限，請先清理"
        ))
        .into());
    }

    let torrent = torrents_repo::insert(state.get_pool(), &info_hash, magnet_uri, created_by, owner_id).await?;
    // spawn 而非 await：add_torrent 解析 magnet metadata 可能卡數分鐘，會拖垮 HTTP 回應（nginx 60s 就斷）
    tokio::spawn(sync_active(state.clone()));
    Ok(torrent)
}

/// 把排隊中（pending）與重啟後中斷（downloading）的任務補進 session，直到達併發上限。
/// 啟動時、新增後、完成/失敗/讓位/刪除後都會呼叫；重複呼叫安全。
/// 回傳 BoxFuture：本函式 spawn `run_torrent`，而它收尾時又回頭 spawn 本函式，
/// opaque future 會變成型別自我參照（不 Send）
pub fn sync_active(state: AppState) -> futures::future::BoxFuture<'static, ()> {
    Box::pin(sync_active_inner(state))
}

async fn sync_active_inner(state: AppState) {
    let manager = state.get_torrents();
    let max_active = max_active(&state);

    let mut slots = manager.active.lock().await;
    if slots.len() >= max_active {
        return;
    }
    // limit 要把已佔位的算進去 —— 它們也在 resumable 清單裡而且排在前面，
    // 只撈 max_active 筆會被自己佔滿、撈不到後面排隊的
    let rows = match torrents_repo::list_resumable(
        state.get_pool(),
        (max_active + slots.len()) as i64,
    )
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("sync_active db error: {e}");
            return;
        }
    };
    let free = max_active - slots.len();
    let to_start: Vec<Torrent> = rows
        .into_iter()
        .filter(|t| !slots.contains_key(&t.id))
        .take(free)
        .collect();

    // 各自 spawn，不在這裡逐一 await：add_torrent 會等 magnet metadata，
    // 序列跑的話第一筆沒 peers 就把後面全卡死
    for row in to_start {
        let id = row.id;
        let task = tokio::spawn(run_torrent(state.clone(), row));
        slots.insert(id, Slot { task, handle: None });
    }
}

/// 有沒有任務排不進併發名額。
/// 這是「解析 metadata 該不該讓位」的唯一判準 —— 沒人在等就沒有理由中斷冷門種子的解析。
/// 查詢失敗時保守回 true（維持讓位行為，寧可多輪替也別卡住整條隊伍）。
async fn queue_pressure(state: &AppState) -> bool {
    match torrents_repo::count_resumable(state.get_pool()).await {
        Ok(total) => total > max_active(state) as i64,
        Err(e) => {
            tracing::warn!("count_resumable failed: {e}");
            true
        }
    }
}

/// 一個任務的完整生命週期：加進 session（解析 metadata，中途可能讓位）→ 監看到完成/失敗。
/// 由 `sync_active` spawn，對應 active map 裡的一格。
async fn run_torrent(state: AppState, row: Torrent) {
    let id = row.id;
    // 記一次嘗試：把自己推到候選排序的隊尾，這輪沒成功時後面的任務才輪得到。
    // DB 掛掉時當第 1 次 —— 寧可多試幾輪，也不要因為記不到帳就把任務判失敗
    let attempt = torrents_repo::mark_attempt(state.get_pool(), id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("torrent {id} mark_attempt failed: {e}");
            1
        });

    let failure = match start_torrent(&state, &row).await {
        // 成功後 watcher 自己收尾（完成/失敗都會清 slot 並補位）
        Ok(handle) => return watch_torrent(state, id, handle).await,
        Err(failure) => failure,
    };

    // 讓位（還有額度）→ 留在 pending 等下一輪；其餘一律判 failed。
    // 兩條路的收尾動作相同：清 slot → 更新 DB → 推播 → 補位
    let retrying =
        matches!(failure, StartFailure::MetadataTimeout) && attempt < MAX_METADATA_ATTEMPTS;
    let reason = match failure {
        StartFailure::MetadataTimeout if retrying => {
            format!("找不到 peers，先讓位給排隊的任務（第 {attempt}/{MAX_METADATA_ATTEMPTS} 次）")
        }
        StartFailure::MetadataTimeout => {
            format!("找不到 peers，連續 {MAX_METADATA_ATTEMPTS} 次讓位後仍無結果")
        }
        StartFailure::Fatal(reason) => reason,
    };

    state.get_torrents().active.lock().await.remove(&id);
    // 這兩個是狀態機轉換（不是快取寫入），失敗不能只用 `let _ =` 吞掉：任務會卡在
    // downloading，list_resumable 一直把它撈出來重試，但 attempt_count 沒遞增，
    // 於是永遠不會被判失敗 —— 變成無限重試迴圈。至少要留下可查的 error log。
    if retrying {
        tracing::warn!("torrent {id} {reason}");
        if let Err(e) = torrents_repo::set_retry_pending(state.get_pool(), id, &reason).await {
            tracing::error!("torrent {id} set_retry_pending failed: {e}（狀態可能卡在 downloading）");
        }
        state.broadcast(
            WsEvent::TorrentRetrying,
            serde_json::json!({ "id": id, "name": row.name, "reason": reason, "attempt": attempt }),
        );
    } else {
        tracing::error!("torrent {id} start failed: {reason}");
        if let Err(e) = torrents_repo::set_failed(state.get_pool(), id, &reason).await {
            tracing::error!("torrent {id} set_failed failed: {e}（狀態可能卡在 downloading）");
        }
        broadcast_failed(&state, id, row.name.as_deref(), &reason);
    }
    tokio::spawn(sync_active(state));
}

async fn start_torrent(
    state: &AppState,
    row: &Torrent,
) -> Result<Arc<ManagedTorrent>, StartFailure> {
    let manager = state.get_torrents();
    let output_dir = manager.output_dir(&row.info_hash);
    let interval = Duration::from_secs(
        setting(
            state,
            "torrent_metadata_timeout_seconds",
            DEFAULT_METADATA_TIMEOUT_SECONDS,
        )
        .clamp(30, 3600) as u64,
    );

    // 解析可能耗上數分鐘，這行是那段期間唯一的痕跡（started 要等解析完才印）
    tracing::info!(
        "torrent {} ({}) resolving metadata, check every {}s",
        row.id,
        row.info_hash,
        interval.as_secs()
    );

    // ⚠ 對 magnet，librqbit 會在 add_torrent 內部解析 metadata（DHT/tracker 找 peers 要 info），
    //   而且它自己沒有逾時 —— 種子沒人做種就是無限期卡在這個 await，任務永遠停在 pending。
    //   逾時必須包在這裡，包在後面的 wait_until_initialized 已經來不及。
    let add_fut = manager.session.add_torrent(
        AddTorrent::from_url(&row.magnet_uri),
        Some(AddTorrentOptions {
            // 重啟 resume：檔案已存在時驗證既有 piece 續抓，不整包重來
            overwrite: true,
            output_folder: Some(output_dir.to_string_lossy().to_string()),
            ..Default::default()
        }),
    );
    tokio::pin!(add_fut);

    // 到點只是「檢查要不要讓位」，不是硬逾時：沒有任務排隊就繼續等下一輪。
    // 對同一個 pinned future 反覆 timeout（而不是重新 add_torrent），已累積的
    // DHT 查詢與半握手的 peer 都留著 —— 冷門種子要的就是不被打斷的時間。
    let response = loop {
        match tokio::time::timeout(interval, &mut add_fut).await {
            Ok(Ok(r)) => break r,
            Ok(Err(e)) => return Err(StartFailure::Fatal(format!("add_torrent failed: {e}"))),
            Err(_) => {
                if !queue_pressure(state).await {
                    tracing::info!(
                        "torrent {} 仍在解析 metadata，無任務排隊等名額，繼續等待",
                        row.id
                    );
                    continue;
                }
                // 有任務排不進名額 → 讓位，本輪放棄（逾時當下極小機率剛好已掛進
                // session，用 info_hash 補刪，別留孤兒）
                purge_by_info_hash(manager, &row.info_hash, false).await;
                return Err(StartFailure::MetadataTimeout);
            }
        }
    };

    let handle = match response {
        AddTorrentResponse::Added(_, handle) => handle,
        AddTorrentResponse::AlreadyManaged(_, handle) => handle,
        AddTorrentResponse::ListOnly(_) => {
            return Err(StartFailure::Fatal(
                "unexpected list-only response".to_string(),
            ))
        }
    };

    // 佔位格補上 handle（即時進度要用）。格子不見 = 解析期間任務被刪掉了
    match manager.active.lock().await.get_mut(&row.id) {
        Some(slot) => slot.handle = Some(handle.clone()),
        None => {
            // 刪不掉就是 librqbit session 裡留了一個沒人管的 torrent（繼續佔頻寬與磁碟），
            // 而外層只會看到「任務已被移除」這個預期內的錯誤 —— 不記就查不出殘留從哪來
            if let Err(e) = manager.session.delete(handle.id().into(), true).await {
                tracing::warn!("torrent {} 佔位格消失後回收 session 失敗: {}", row.id, e);
            }
            return Err(StartFailure::Fatal(
                "任務已於啟動期間被移除".to_string(),
            ));
        }
    }
    tracing::info!("torrent {} ({}) started", row.id, row.info_hash);
    Ok(handle)
}

/// 拿到 metadata 之後的監看：等初始化 → metadata 落 DB → 進度推播 → 完成/失敗收尾
async fn watch_torrent(state: AppState, id: i32, handle: Arc<ManagedTorrent>) {
    // 1. 等初始化完成（metadata 已在 start_torrent 拿到，這裡等的是既有檔案的 piece 驗證）
    match tokio::time::timeout(INIT_TIMEOUT, handle.wait_until_initialized()).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            finish_failed(&state, id, &handle, &format!("初始化失敗: {e}")).await;
            return;
        }
        Err(_) => {
            finish_failed(&state, id, &handle, "初始化逾時（既有檔案驗證未完成）").await;
            return;
        }
    }

    // 2. metadata 落 DB
    let name = handle.name().unwrap_or_else(|| format!("torrent-{id}"));
    let files: Vec<TorrentFile> = match handle.with_metadata(|m| {
        m.file_infos
            .iter()
            .enumerate()
            .map(|(index, f)| TorrentFile {
                index,
                path: f.relative_filename.to_string_lossy().to_string(),
                size: f.len,
            })
            .collect()
    }) {
        Ok(files) => files,
        Err(e) => {
            finish_failed(&state, id, &handle, &format!("讀取 metadata 失敗: {e}")).await;
            return;
        }
    };
    let total_size: i64 = files.iter().map(|f| f.size as i64).sum();
    let files_json = match serde_json::to_value(&files) {
        Ok(v) => v,
        Err(e) => {
            finish_failed(&state, id, &handle, &format!("序列化檔案清單失敗: {e}")).await;
            return;
        }
    };
    if let Err(e) =
        torrents_repo::set_downloading_metadata(state.get_pool(), id, &name, total_size, &files_json)
            .await
    {
        tracing::error!("torrent {id} metadata db update failed: {e}");
    }

    // 3. 輪詢進度：5 秒一次、有變動才推播
    let mut last_percent = -1.0_f64;
    loop {
        let stats = handle.stats();

        if let Some(error) = stats.error {
            finish_failed(&state, id, &handle, &error).await;
            return;
        }

        if stats.finished {
            finish_completed(&state, id, &handle, &name, total_size).await;
            return;
        }

        let percent = if stats.total_bytes > 0 {
            (stats.progress_bytes as f64 / stats.total_bytes as f64 * 10000.0).round() / 100.0
        } else {
            0.0
        };
        if (percent - last_percent).abs() > f64::EPSILON {
            last_percent = percent;
            let (down_speed, peers) = stats
                .live
                .as_ref()
                .map(|l| (l.download_speed.to_string(), l.snapshot.peer_stats.live))
                .unwrap_or_default();
            state.broadcast(
                WsEvent::TorrentProgress,
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "progress": percent,
                    "progress_bytes": stats.progress_bytes,
                    "total_bytes": stats.total_bytes,
                    "down_speed": down_speed,
                    "peers": peers,
                }),
            );
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// 完成收尾：從 session 移除（停止做種、保留檔案）→ DB → 推播 → 補位
async fn finish_completed(
    state: &AppState,
    id: i32,
    handle: &Arc<ManagedTorrent>,
    name: &str,
    total_size: i64,
) {
    remove_from_session(state, id, handle, false).await;
    if let Err(e) = torrents_repo::set_completed(state.get_pool(), id).await {
        tracing::error!("torrent {id} set_completed failed: {e}");
    }
    tracing::info!("torrent {id} ({name}) completed");
    state.broadcast(
        WsEvent::TorrentCompleted,
        serde_json::json!({ "id": id, "name": name, "total_size": total_size }),
    );
    {
        let settings = state.get_settings();
        let subject = format!("Torrent 下載完成：{name}");
        let body = format!(
            "任務 #{id}\n名稱：{name}\n大小：{}\n\n到後台 /admin/torrents 產生下載連結取檔。",
            format_size(total_size)
        );
        tokio::spawn(async move {
            // 這封是**一次性**的：torrent 沒有「未通知」欄可以留著下輪補寄，寄失敗就是
            // 失敗（`send_to` 已記 ERROR）。使用者仍看得到任務變 completed，只是少一封信。
            let _ = crate::services::email::send_notification(&settings, &subject, body).await;
        });
    }
    // spawn 而非 await：斷開 watch_torrent ↔ sync_active 的遞迴，避免 future Send 自我參照
    tokio::spawn(sync_active(state.clone()));
}

fn format_size(bytes: i64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else {
        format!("{:.1} MB", b / MB)
    }
}

/// 失敗收尾：從 session 移除（保留已下載部分供重試續抓）→ DB → 推播 → 補位
async fn finish_failed(state: &AppState, id: i32, handle: &Arc<ManagedTorrent>, reason: &str) {
    remove_from_session(state, id, handle, false).await;
    let name = handle.name();
    if let Err(e) = torrents_repo::set_failed(state.get_pool(), id, reason).await {
        tracing::error!("torrent {id} set_failed failed: {e}");
    }
    tracing::warn!("torrent {id} failed: {reason}");
    broadcast_failed(state, id, name.as_deref(), reason);
    tokio::spawn(sync_active(state.clone()));
}

fn broadcast_failed(state: &AppState, id: i32, name: Option<&str>, reason: &str) {
    state.broadcast(
        WsEvent::TorrentFailed,
        serde_json::json!({ "id": id, "name": name, "reason": reason }),
    );
}

/// 從 librqbit session 移除；失敗只 warn，收尾流程不因此中斷
async fn session_delete(manager: &TorrentManager, target: TorrentIdOrHash, delete_files: bool) {
    if let Err(e) = manager.session.delete(target, delete_files).await {
        tracing::warn!("session delete {target:?} failed: {e}");
    }
}

async fn remove_from_session(
    state: &AppState,
    id: i32,
    handle: &Arc<ManagedTorrent>,
    delete_files: bool,
) {
    let manager = state.get_torrents();
    // 移掉的 Slot 帶著本 task 自己的 JoinHandle，drop 只是 detach，不會中止自己
    manager.active.lock().await.remove(&id);
    session_delete(manager, handle.id().into(), delete_files).await;
}

/// 沒有 handle 時的兜底清理（啟動 task 被 abort、或讓位時剛好已掛進 session），
/// 用 info_hash 找殘留。**先查存在再刪**：讓位路徑每次都會走這裡，絕大多數是 no-op，
/// 少了這道檢查每次讓位都會噴一行 warn。
async fn purge_by_info_hash(manager: &TorrentManager, info_hash: &str, delete_files: bool) {
    let Ok(target) = TorrentIdOrHash::parse(info_hash) else {
        return;
    };
    if manager.session.get(target).is_some() {
        session_delete(manager, target, delete_files).await;
    }
}

/// 分頁列出任務（super_admin 看全部，其餘只看自己的）
pub async fn list(
    state: &AppState,
    actor: &AuthenticatedUser,
    status: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Torrent>, AppError> {
    torrents_repo::list(state.get_pool(), status, actor.owner_filter(), limit, offset).await
}

/// 每支「指定單一任務」的端點共用的擁有者檢查。
///
/// **放在 service 而不是 route**：`torrents` 的資料隔離規則（非擁有者一律 404、
/// super_admin 全可）是業務規則，而 route 端有 4 支要重複同一行；漏掉一支不會有
/// 任何徵兆，只會變成一個能讀別人任務的側門。
async fn ensure_owner(state: &AppState, actor: &AuthenticatedUser, id: i32) -> Result<(), AppError> {
    actor.require_owner(torrents_repo::get_owner(state.get_pool(), id).await?)
}

/// 重設 failed / completed 任務為 pending 重跑
pub async fn reset_pending(state: &AppState, actor: &AuthenticatedUser, id: i32) -> Result<(), AppError> {
    ensure_owner(state, actor, id).await?;
    if !torrents_repo::reset_pending(state.get_pool(), id).await? {
        // id 不存在 → 404；存在但下載中 → 409
        torrents_repo::get_by_id(state.get_pool(), id).await?;
        return Err(RequestError::Conflict("任務進行中，無法重設".to_string()).into());
    }
    tokio::spawn(sync_active(state.clone()));
    Ok(())
}

/// 刪除任務（端點入口，帶擁有者檢查）
pub async fn delete(state: &AppState, actor: &AuthenticatedUser, id: i32) -> Result<(), AppError> {
    ensure_owner(state, actor, id).await?;
    delete_by_id(state, id).await
}

/// 刪除任務：session 停掉 → DB 刪除 → 磁碟清理 → 補位。
/// **不做擁有者檢查** —— 呼叫端要嘛已檢查（`delete`），要嘛沒有身分可檢查
/// （`cleanup_expired` 是排程 job）。
async fn delete_by_id(state: &AppState, id: i32) -> Result<(), AppError> {
    let manager = state.get_torrents();
    // 先抽走佔位格並中止 task —— 還卡在解析 metadata（尚無 handle）的任務也要刪得掉
    let slot = manager.active.lock().await.remove(&id);
    if let Some(slot) = &slot {
        slot.task.abort();
    }
    let info_hash = torrents_repo::delete(state.get_pool(), id).await?;
    match slot.and_then(|s| s.handle) {
        Some(handle) => session_delete(manager, handle.id().into(), true).await,
        // 還在解析、沒有 handle：abort 前可能剛好已掛進 session，用 info_hash 兜底
        None => purge_by_info_hash(manager, &info_hash, true).await,
    }
    let dir = manager.output_dir(&info_hash);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!("torrent {id} remove dir {} failed: {e}", dir.display());
        }
    }
    tokio::spawn(sync_active(state.clone()));
    Ok(())
}

/// 任務詳情：DB row + 進行中任務附上即時進度
pub async fn detail(state: &AppState, actor: &AuthenticatedUser, id: i32) -> Result<serde_json::Value, AppError> {
    ensure_owner(state, actor, id).await?;
    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    let mut value = serde_json::to_value(&torrent)?;

    if let Some(handle) = state.get_torrents().get_handle(id).await {
        let stats = handle.stats();
        let percent = if stats.total_bytes > 0 {
            (stats.progress_bytes as f64 / stats.total_bytes as f64 * 10000.0).round() / 100.0
        } else {
            0.0
        };
        let (down_speed, peers) = stats
            .live
            .as_ref()
            .map(|l| (l.download_speed.to_string(), l.snapshot.peer_stats.live))
            .unwrap_or_default();
        value["live"] = serde_json::json!({
            "progress": percent,
            "progress_bytes": stats.progress_bytes,
            "total_bytes": stats.total_bytes,
            "down_speed": down_speed,
            "peers": peers,
        });
    }

    Ok(value)
}

/// 產生所有檔案的短效簽名下載連結
pub async fn create_download_links(
    state: &AppState,
    actor: &AuthenticatedUser,
    id: i32,
) -> Result<Vec<DownloadLink>, AppError> {
    ensure_owner(state, actor, id).await?;
    let issuer_id = actor.id;
    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    if torrent.status != STATUS_COMPLETED {
        return Err(RequestError::Conflict("任務尚未完成，無法下載".to_string()).into());
    }
    let files: Vec<TorrentFile> = torrent
        .files
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();

    let ttl_minutes = setting(state, "torrent_link_ttl_minutes", DEFAULT_LINK_TTL_MINUTES).max(1);
    let expires_at = Utc::now() + Duration::from_secs(ttl_minutes as u64 * 60);
    let secret = &state.get_config().jwt_secret;

    files
        .into_iter()
        .map(|f| {
            let claims = TorrentDownloadClaims {
                exp: expires_at.timestamp() as usize,
                purpose: DOWNLOAD_TOKEN_PURPOSE.to_string(),
                sub: issuer_id.to_string(),
                torrent_id: id,
                file_index: f.index,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            )
            .map_err(|e| SystemError::Internal(format!("簽發下載 token 失敗: {e}")))?;
            Ok(DownloadLink {
                url: format!("/admin/torrents/{id}/files/{}?token={token}", f.index),
                file_index: f.index,
                path: f.path,
                size: f.size,
                expires_at,
            })
        })
        .collect()
}

/// 驗證下載 token 並解析出實體檔案路徑（含 path traversal 防護）
pub async fn resolve_download_file(
    state: &AppState,
    id: i32,
    file_index: usize,
    token: &str,
) -> Result<(PathBuf, String), AppError> {
    let claims = decode::<TorrentDownloadClaims>(
        token,
        &DecodingKey::from_secret(state.get_config().jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::AuthError(crate::errors::AuthError::InvalidToken))?
    .claims;

    if claims.purpose != DOWNLOAD_TOKEN_PURPOSE
        || claims.torrent_id != id
        || claims.file_index != file_index
    {
        return Err(AppError::AuthError(crate::errors::AuthError::InvalidToken));
    }

    // 發行者 id（token sub）
    let issuer_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::AuthError(crate::errors::AuthError::InvalidToken))?;

    // 即時重查發行者身分（同 auth middleware 的載入點）— 權限被拔掉或帳號被刪，
    // 已發出的連結立即失效
    let identity =
        crate::services::auth::load_identity(state.get_pool(), state.get_redis_pool(), issuer_id)
            .await?
            .ok_or(AppError::AuthError(crate::errors::AuthError::Forbidden))?;
    if !identity
        .permissions
        .iter()
        .any(|p| p == crate::structs::roles::Perm::TorrentRead.as_str())
    {
        return Err(AppError::AuthError(crate::errors::AuthError::Forbidden));
    }

    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    if torrent.status != STATUS_COMPLETED {
        return Err(RequestError::Conflict("任務尚未完成，無法下載".to_string()).into());
    }
    let files: Vec<TorrentFile> = torrent
        .files
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let file = files
        .into_iter()
        .find(|f| f.index == file_index)
        .ok_or(RequestError::NotFound)?;

    let manager = state.get_torrents();
    let dir = manager.output_dir(&torrent.info_hash);
    let path = dir.join(&file.path);

    // canonicalize 後確認還在任務目錄底下，擋 metadata 帶 ../ 的惡意路徑
    let canonical = tokio::fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::from(RequestError::NotFound))?;
    let canonical_dir = tokio::fs::canonicalize(&dir)
        .await
        .map_err(|_| AppError::from(RequestError::NotFound))?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(RequestError::NotFound.into());
    }

    let filename = std::path::Path::new(&file.path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("torrent-{id}-{file_index}"));

    Ok((canonical, filename))
}

/// 儲存空間概況：TORRENT_PATH 所在檔案系統的實際剩餘 + torrent 配額用量
pub async fn storage_stats(state: &AppState) -> Result<serde_json::Value, AppError> {
    let manager = state.get_torrents();
    let (disk_total, disk_available) = disk_space(manager.base_path())
        .map_err(|e| SystemError::Internal(format!("statvfs failed: {e}")))?;

    let used = torrents_repo::total_size_sum(state.get_pool()).await?;
    let max_bytes = setting(state, "torrent_max_total_size_gb", DEFAULT_MAX_TOTAL_SIZE_GB)
        .saturating_mul(1024 * 1024 * 1024);

    Ok(serde_json::json!({
        "disk": {
            "total_bytes": disk_total,
            "available_bytes": disk_available,
        },
        "torrent": {
            "used_bytes": used,
            "max_bytes": max_bytes,
        },
    }))
}

/// statvfs 查路徑所在檔案系統的 (總容量, 非 root 可用容量)，單位 bytes
fn disk_space(path: &std::path::Path) -> std::io::Result<(u64, u64)> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let total = stat.f_blocks as u64 * stat.f_frsize as u64;
    let available = stat.f_bavail as u64 * stat.f_frsize as u64;
    Ok((total, available))
}

/// 排程：清除逾期任務（completed 超過保留天數 / failed 同），刪 DB + 磁碟
pub async fn cleanup_expired(state: &AppState) -> Result<(), AppError> {
    let retention_days = setting(state, "torrent_retention_days", 7i64);
    let expired = torrents_repo::list_expired(state.get_pool(), retention_days).await?;
    for torrent in expired {
        tracing::info!(
            "cleanup expired torrent {} ({})",
            torrent.id,
            torrent.name.as_deref().unwrap_or("-")
        );
        if let Err(e) = delete_by_id(state, torrent.id).await {
            tracing::error!("cleanup torrent {} failed: {e}", torrent.id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TorrentManager::new()` 真的建得起來，而且 socket 真的是 IPv4 —— **`#[ignore]`，CI 不跑**。
    ///
    /// 為什麼要有這條:整條 torrent 路徑在 CI 是零覆蓋，而 `new()` 裡的
    /// `.expect("failed to create torrent session")` 在**啟動路徑**上 —— 它炸掉不是
    /// 「torrent 功能壞了」，是**整個 backend 開不起來**。dependabot 只會改版號，
    /// 而 librqbit 8→9 的預設就從 v4 變成 dualstack（見 `new()` 裡的長註解）。
    ///
    /// 為什麼不只斷言「不 panic」:librqbit 綁 `[::]` 在**開發機**上是會成功的（本機有
    /// IPv6），所以只驗 `new()` 回得來等於什麼都沒驗。要驗的是**綁出來的 family**。
    ///
    /// 為什麼 `#[ignore]`:這條會建真的 DHT socket 並往外 bootstrap。放進 CI 等於讓一個
    /// 網路依賴決定 test job 綠不綠。
    ///
    /// 升 librqbit 大版時在本機跑一次:
    /// ```text
    /// cargo test --locked -- --ignored torrent_session_builds --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "會建真的 DHT socket 並對外 bootstrap;升 librqbit 大版時手動跑"]
    async fn torrent_session_builds() {
        let dir = std::env::temp_dir().join("kawa-torrent-session-test");
        // SAFETY: 單執行緒測試、只設一次，沒有其他 thread 同時讀 env
        unsafe { std::env::set_var("TORRENT_PATH", &dir) };
        let manager = TorrentManager::new().await;
        assert_eq!(manager.base_path(), dir.as_path());

        let v4 = own_udp_sockets("/proc/self/net/udp");
        let v6 = own_udp_sockets("/proc/self/net/udp6");
        // 失敗時看得到全貌，才不必重跑一次加 print
        println!("own udp4: {v4:#?}\nown udp6: {v6:#?}");

        // DHT 吃 `ipv4_only`，所以一定有一顆 v4 UDP socket。一顆都沒有 = 上游又改了預設。
        assert!(!v4.is_empty(), "沒有任何 v4 UDP socket，`ipv4_only` 疑似沒生效");

        // LSD 不吃 `ipv4_only`，靠 `disable_local_service_discovery` 關掉。
        // 6771 = BitTorrent LSD 的多播埠;它出現代表那個開關失效或上游改名。
        assert!(
            !v6.iter().any(|s| s.port == 6771),
            "LSD 還活著（[::]:6771），`disable_local_service_discovery` 沒生效"
        );

        // 剩下允許存在的 v6 socket 只有 UDP tracker client 那顆（`[::]:0` 寫死、dualstack，
        // 功能正常）。多於一顆就是又有新東西不吃 `ipv4_only`，回去讀 `new()` 的註解。
        assert!(
            v6.len() <= 1,
            "多出沒預期的 v6 socket，可能有新元件不吃 `ipv4_only`: {v6:#?}"
        );

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[derive(Debug)]
    struct OwnSocket {
        port: u16,
        /// 原始的 `hex_addr:hex_port`。只在 assert 失敗訊息（`Debug`）裡出現，
        /// dead code 分析看不到 `Debug` 的用法，所以要 `expect`。
        #[expect(dead_code)]
        local: String,
    }

    /// 本行程自己持有的 UDP socket。
    /// `/proc/self/net/udp*` 是**整個 netns** 的（會有別的行程），所以要跟
    /// `/proc/self/fd` 的 `socket:[inode]` 交叉比對才算是「自己的」。
    fn own_udp_sockets(table: &str) -> Vec<OwnSocket> {
        let mut own = std::collections::HashSet::new();
        for entry in std::fs::read_dir("/proc/self/fd")
            .expect("read /proc/self/fd")
            .flatten()
        {
            let inode = std::fs::read_link(entry.path()).ok().and_then(|target| {
                target
                    .to_string_lossy()
                    .strip_prefix("socket:[")
                    .and_then(|s| s.strip_suffix(']'))
                    .and_then(|s| s.parse::<u64>().ok())
            });
            if let Some(inode) = inode {
                own.insert(inode);
            }
        }

        std::fs::read_to_string(table)
            .unwrap_or_default()
            .lines()
            .skip(1) // 標頭
            .filter_map(|line| {
                let cols: Vec<&str> = line.split_whitespace().collect();
                // 欄位:1 = local_address（`hex_addr:hex_port`），9 = inode
                let local = cols.get(1)?;
                let inode: u64 = cols.get(9)?.parse().ok()?;
                if !own.contains(&inode) {
                    return None;
                }
                let port = u16::from_str_radix(local.split(':').nth(1)?, 16).ok()?;
                Some(OwnSocket {
                    port,
                    local: (*local).to_string(),
                })
            })
            .collect()
    }
}
