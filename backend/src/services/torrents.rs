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
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, Magnet, ManagedTorrent, Session};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;

/// metadata 解析逾時（抓不到 peers/tracker 就放棄）
const METADATA_TIMEOUT: Duration = Duration::from_secs(600);
/// 進度輪詢間隔
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// 下載連結效期預設值（分鐘）— 可由 app_settings.torrent_link_ttl_minutes 熱更新
const DEFAULT_LINK_TTL_MINUTES: i64 = 180;

const DEFAULT_MAX_ACTIVE: usize = 2;
const DEFAULT_MAX_TOTAL_SIZE_GB: i64 = 20;

/// torrent session 與進行中任務的 handle 對照表。
/// 進度不落 DB — 即時資訊一律從 handle 讀。
pub struct TorrentManager {
    session: Arc<Session>,
    active: Mutex<HashMap<i32, Arc<ManagedTorrent>>>,
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
        self.active.lock().await.get(&id).cloned()
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
    sync_active(state.clone()).await;
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

    let rows = {
        let active = manager.active.lock().await;
        if active.len() >= max_active {
            return;
        }
        match torrents_repo::list_resumable(state.get_pool(), max_active as i64).await {
            Ok(rows) => rows
                .into_iter()
                .filter(|t| !active.contains_key(&t.id))
                .take(max_active - active.len())
                .collect::<Vec<_>>(),
            Err(e) => {
                tracing::error!("sync_active db error: {e}");
                return;
            }
        }
    };

    for row in rows {
        if let Err(e) = start_torrent(&state, &row).await {
            tracing::error!("torrent {} start failed: {e}", row.id);
            let _ = torrents_repo::set_failed(state.get_pool(), row.id, &e.to_string()).await;
            broadcast_failed(&state, row.id, row.name.as_deref(), &e.to_string());
        }
    }
}

async fn start_torrent(state: &AppState, row: &Torrent) -> Result<(), AppError> {
    let manager = state.get_torrents();
    let output_dir = manager.output_dir(&row.info_hash);

    let response = manager
        .session
        .add_torrent(
            AddTorrent::from_url(&row.magnet_uri),
            Some(AddTorrentOptions {
                // 重啟 resume：檔案已存在時驗證既有 piece 續抓，不整包重來
                overwrite: true,
                output_folder: Some(output_dir.to_string_lossy().to_string()),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| SystemError::Internal(format!("add_torrent failed: {e}")))?;

    let handle = match response {
        AddTorrentResponse::Added(_, handle) => handle,
        AddTorrentResponse::AlreadyManaged(_, handle) => handle,
        AddTorrentResponse::ListOnly(_) => {
            return Err(SystemError::Internal("unexpected list-only response".to_string()).into())
        }
    };

    {
        // 併發 sync_active 防護：已有 watcher 就不重複 spawn
        let mut active = manager.active.lock().await;
        if active.contains_key(&row.id) {
            return Ok(());
        }
        active.insert(row.id, handle.clone());
    }
    tokio::spawn(watch_torrent(state.clone(), row.id, handle));
    tracing::info!("torrent {} ({}) started", row.id, row.info_hash);
    Ok(())
}

/// 單一任務的生命週期監看：metadata → 進度推播 → 完成/失敗收尾
async fn watch_torrent(state: AppState, id: i32, handle: Arc<ManagedTorrent>) {
    // 1. 等 metadata（DHT/tracker 解析），逾時放棄
    match tokio::time::timeout(METADATA_TIMEOUT, handle.wait_until_initialized()).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            finish_failed(&state, id, &handle, &format!("初始化失敗: {e}")).await;
            return;
        }
        Err(_) => {
            finish_failed(&state, id, &handle, "metadata 解析逾時（找不到 peers）").await;
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
    manager.active.lock().await.remove(&id);
    if let Err(e) = manager.session.delete(handle.id().into(), delete_files).await {
        tracing::warn!("torrent {id} session delete failed: {e}");
    }
}

/// 重設 failed / completed 任務為 pending 重跑
pub async fn reset_pending(state: &AppState, id: i32) -> Result<(), AppError> {
    if !torrents_repo::reset_pending(state.get_pool(), id).await? {
        // id 不存在 → 404；存在但下載中 → 409
        torrents_repo::get_by_id(state.get_pool(), id).await?;
        return Err(RequestError::Conflict("任務進行中，無法重設".to_string()).into());
    }
    sync_active(state.clone()).await;
    Ok(())
}

/// 刪除任務：session 停掉 → DB 刪除 → 磁碟清理 → 補位
pub async fn delete(state: &AppState, id: i32) -> Result<(), AppError> {
    let manager = state.get_torrents();
    if let Some(handle) = manager.get_handle(id).await {
        remove_from_session(state, id, &handle, true).await;
    }
    let info_hash = torrents_repo::delete(state.get_pool(), id).await?;
    let dir = manager.output_dir(&info_hash);
    if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!("torrent {id} remove dir {} failed: {e}", dir.display());
        }
    }
    sync_active(state.clone()).await;
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
