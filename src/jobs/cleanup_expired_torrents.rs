use crate::{services::torrents as torrents_service, state::AppState};

pub async fn run(state: AppState) {
    if let Err(e) = torrents_service::cleanup_expired(&state).await {
        tracing::error!("cleanup_expired_torrents failed: {e}");
    }
}
