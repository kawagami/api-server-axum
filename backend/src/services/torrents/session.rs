use crate::state::AppState;
use librqbit::{api::TorrentIdOrHash, ManagedTorrent};
use std::sync::Arc;
use super::manager::TorrentManager;

/// 從 librqbit session 移除；失敗只 warn，收尾流程不因此中斷
pub(super) async fn session_delete(manager: &TorrentManager, target: TorrentIdOrHash, delete_files: bool) {
    if let Err(e) = manager.session.delete(target, delete_files).await {
        tracing::warn!("session delete {target:?} failed: {e}");
    }
}

pub(super) async fn remove_from_session(
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
pub(super) async fn purge_by_info_hash(manager: &TorrentManager, info_hash: &str, delete_files: bool) {
    let Ok(target) = TorrentIdOrHash::parse(info_hash) else {
        return;
    };
    if manager.session.get(target).is_some() {
        session_delete(manager, target, delete_files).await;
    }
}
