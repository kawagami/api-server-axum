use crate::{services::torrents as torrents_service, state::AppState};

pub async fn run(state: AppState) {
    if let Err(e) = torrents_service::cleanup_expired(&state).await {
        tracing::error!("cleanup_expired_torrents failed: {e}");
    }
    // 保險：補位平常靠事件驅動（新增/完成/失敗/刪除），
    // 那些觸發點若剛好遇上 DB 抽風就漏掉了，排隊中的任務會一直躺在 pending
    torrents_service::sync_active(state).await;
}
