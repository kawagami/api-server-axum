use crate::{repositories::stocks::sync_buyback_periods_to_pending, state::AppState};

pub async fn run(state: AppState) {
    match sync_buyback_periods_to_pending(state.get_pool()).await {
        Ok(n) => tracing::info!("sync_buyback_to_pending inserted {} rows", n),
        Err(e) => tracing::error!("sync_buyback_to_pending fail: {}", e),
    }
}
