use crate::{
    errors::{AppError, RequestError},
    repositories::torrents as torrents_repo,
    state::AppState,
    structs::{auth::AuthenticatedUser, pagination::Paginated, torrents::Torrent},
};
use librqbit::Magnet;
use super::lifecycle::sync_active;
use super::manager::{DEFAULT_MAX_TOTAL_SIZE_GB, setting};
use super::session::{purge_by_info_hash, session_delete};

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
pub(super) async fn ensure_owner(state: &AppState, actor: &AuthenticatedUser, id: i32) -> Result<(), AppError> {
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
