use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::torrents as torrents_repo,
    state::AppState,
    structs::{
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
    Session,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};

/// metadata 解析逾時預設值（秒）— 可由 app_settings.torrent_metadata_timeout_seconds 熱更新。
/// 活著的冷門種子通常一兩分鐘內就能從 DHT 撈到 peer，撐更久多半是死種，
/// 與其乾等不如讓後面排隊的先試（逾時會排到隊尾，額度用完才判 failed）。
const DEFAULT_METADATA_TIMEOUT_SECONDS: i64 = 180;
/// metadata 逾時的重試額度：連續這麼多輪都找不到 peers 才判 failed
const MAX_METADATA_ATTEMPTS: i32 = 3;
/// 初始化逾時 — metadata 到手後驗證磁碟既有 piece（純本地 IO + SHA-1，大檔在小機器上會久）
const INIT_TIMEOUT: Duration = Duration::from_secs(1800);
/// 進度輪詢間隔
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// 下載連結效期預設值（分鐘）— 可由 app_settings.torrent_link_ttl_minutes 熱更新
const DEFAULT_LINK_TTL_MINUTES: i64 = 180;

const DEFAULT_MAX_ACTIVE: usize = 2;
const DEFAULT_MAX_TOTAL_SIZE_GB: i64 = 20;

/// 啟動失敗的分類 —— metadata 逾時還有重試機會，其他錯誤直接判 failed
enum StartFailure {
    /// 這一輪沒在時限內找到 peers（額度未用完就只是排到隊尾）
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
        let session = Session::new(base_path.clone())
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

fn settings_usize(state: &AppState, key: &str, default: usize) -> usize {
    state
        .get_settings()
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

fn settings_i64(state: &AppState, key: &str, default: i64) -> i64 {
    state
        .get_settings()
        .get(key)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
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

    let max_bytes = settings_i64(state, "torrent_max_total_size_gb", DEFAULT_MAX_TOTAL_SIZE_GB)
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
/// 啟動時、新增後、完成/失敗/刪除後都會呼叫；重複呼叫安全。
/// 回傳 BoxFuture：與 watch_torrent 互相遞迴，opaque future 會造成 Send 自我參照
pub fn sync_active(state: AppState) -> futures::future::BoxFuture<'static, ()> {
    Box::pin(sync_active_inner(state))
}

async fn sync_active_inner(state: AppState) {
    let manager = state.get_torrents();
    let max_active = settings_usize(&state, "torrent_max_active", DEFAULT_MAX_ACTIVE);

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
    let max_active = settings_usize(state, "torrent_max_active", DEFAULT_MAX_ACTIVE) as i64;
    match torrents_repo::count_resumable(state.get_pool()).await {
        Ok(total) => total > max_active,
        Err(e) => {
            tracing::warn!("count_resumable failed: {e}");
            true
        }
    }
}

/// 一個任務的完整生命週期：加進 session（含 metadata 逾時）→ 監看到完成/失敗。
/// 由 `sync_active` spawn，對應 active map 裡的一格。
async fn run_torrent(state: AppState, row: Torrent) {
    let id = row.id;
    // 記一次嘗試：把自己推到候選排序的隊尾，這輪沒成功時後面的任務才輪得到
    let attempt = torrents_repo::mark_attempt(state.get_pool(), id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("torrent {id} mark_attempt failed: {e}");
            1
        });

    let handle = match start_torrent(&state, &row).await {
        Ok(handle) => handle,
        // 還有額度就留在 pending 等下一輪（此時候選排序已把它排到最後）
        Err(StartFailure::MetadataTimeout) if attempt < MAX_METADATA_ATTEMPTS => {
            let reason = format!(
                "找不到 peers，先讓位給排隊的任務（第 {attempt}/{MAX_METADATA_ATTEMPTS} 次）"
            );
            tracing::warn!("torrent {id} {reason}");
            state.get_torrents().active.lock().await.remove(&id);
            let _ = torrents_repo::set_retry_pending(state.get_pool(), id, &reason).await;
            state.broadcast(
                WsEvent::TorrentRetrying,
                serde_json::json!({ "id": id, "name": row.name, "reason": reason, "attempt": attempt }),
            );
            tokio::spawn(sync_active(state.clone()));
            return;
        }
        Err(failure) => {
            let reason = match failure {
                StartFailure::MetadataTimeout => {
                    format!("找不到 peers，連續 {MAX_METADATA_ATTEMPTS} 次讓位後仍無結果")
                }
                StartFailure::Fatal(reason) => reason,
            };
            tracing::error!("torrent {id} start failed: {reason}");
            state.get_torrents().active.lock().await.remove(&id);
            let _ = torrents_repo::set_failed(state.get_pool(), id, &reason).await;
            broadcast_failed(&state, id, row.name.as_deref(), &reason);
            tokio::spawn(sync_active(state.clone()));
            return;
        }
    };
    watch_torrent(state, id, handle).await;
}

async fn start_torrent(
    state: &AppState,
    row: &Torrent,
) -> Result<Arc<ManagedTorrent>, StartFailure> {
    let manager = state.get_torrents();
    let output_dir = manager.output_dir(&row.info_hash);
    let interval = Duration::from_secs(
        settings_i64(
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
            let _ = manager.session.delete(handle.id().into(), true).await;
            return Err(StartFailure::Fatal(
                "任務已於啟動期間被移除".to_string(),
            ));
        }
    }
    tracing::info!("torrent {} ({}) started", row.id, row.info_hash);
    Ok(handle)
}

/// 單一任務的生命週期監看：metadata → 進度推播 → 完成/失敗收尾
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
            crate::services::email::send_notification(&settings, &subject, body).await;
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

async fn remove_from_session(
    state: &AppState,
    id: i32,
    handle: &Arc<ManagedTorrent>,
    delete_files: bool,
) {
    let manager = state.get_torrents();
    // 移掉的 Slot 帶著本 task 自己的 JoinHandle，drop 只是 detach，不會中止自己
    manager.active.lock().await.remove(&id);
    if let Err(e) = manager.session.delete(handle.id().into(), delete_files).await {
        tracing::warn!("torrent {id} session delete failed: {e}");
    }
}

/// 沒有 handle 時的兜底清理（啟動 task 被 abort / 逾時），用 info_hash 找 session 內的殘留
async fn purge_by_info_hash(manager: &TorrentManager, info_hash: &str, delete_files: bool) {
    let Ok(target) = TorrentIdOrHash::parse(info_hash) else {
        return;
    };
    if manager.session.get(target).is_none() {
        return;
    }
    if let Err(e) = manager.session.delete(target, delete_files).await {
        tracing::warn!("torrent {info_hash} session delete by hash failed: {e}");
    }
}

/// 重設 failed / completed 任務為 pending 重跑
pub async fn reset_pending(state: &AppState, id: i32) -> Result<(), AppError> {
    if !torrents_repo::reset_pending(state.get_pool(), id).await? {
        // id 不存在 → 404；存在但下載中 → 409
        torrents_repo::get_by_id(state.get_pool(), id).await?;
        return Err(RequestError::Conflict("任務進行中，無法重設".to_string()).into());
    }
    tokio::spawn(sync_active(state.clone()));
    Ok(())
}

/// 刪除任務：session 停掉 → DB 刪除 → 磁碟清理 → 補位
pub async fn delete(state: &AppState, id: i32) -> Result<(), AppError> {
    let manager = state.get_torrents();
    // 先抽走佔位格並中止 task —— 還卡在解析 metadata（尚無 handle）的任務也要刪得掉
    let slot = manager.active.lock().await.remove(&id);
    if let Some(slot) = &slot {
        slot.task.abort();
    }
    let info_hash = torrents_repo::delete(state.get_pool(), id).await?;
    match slot.and_then(|s| s.handle) {
        Some(handle) => {
            if let Err(e) = manager.session.delete(handle.id().into(), true).await {
                tracing::warn!("torrent {id} session delete failed: {e}");
            }
        }
        // abort 前可能剛好已掛進 session，用 info_hash 兜底
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
pub async fn detail(state: &AppState, id: i32) -> Result<serde_json::Value, AppError> {
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
    id: i32,
    issuer_id: i64,
) -> Result<Vec<DownloadLink>, AppError> {
    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    if torrent.status != STATUS_COMPLETED {
        return Err(RequestError::Conflict("任務尚未完成，無法下載".to_string()).into());
    }
    let files: Vec<TorrentFile> = torrent
        .files
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();

    let ttl_minutes = settings_i64(state, "torrent_link_ttl_minutes", DEFAULT_LINK_TTL_MINUTES).max(1);
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

    // 即時重查發行者權限（Redis 快取，同 auth middleware）— 權限被拔掉，已發出的連結立即失效
    let permissions = match crate::repositories::redis::get_user_permissions(
        state.get_redis_pool(),
        issuer_id,
    )
    .await?
    {
        Some(perms) => perms,
        None => {
            let perms = crate::repositories::roles::get_user_permission_strings_by_id(
                state.get_pool(),
                issuer_id,
            )
            .await?;
            let _ = crate::repositories::redis::set_user_permissions(
                state.get_redis_pool(),
                issuer_id,
                &perms,
            )
            .await;
            perms
        }
    };
    if !permissions
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
    let max_bytes = settings_i64(state, "torrent_max_total_size_gb", DEFAULT_MAX_TOTAL_SIZE_GB)
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
    let retention_days = settings_i64(state, "torrent_retention_days", 7);
    let expired = torrents_repo::list_expired(state.get_pool(), retention_days).await?;
    for torrent in expired {
        tracing::info!(
            "cleanup expired torrent {} ({})",
            torrent.id,
            torrent.name.as_deref().unwrap_or("-")
        );
        if let Err(e) = delete(state, torrent.id).await {
            tracing::error!("cleanup torrent {} failed: {e}", torrent.id);
        }
    }
    Ok(())
}
