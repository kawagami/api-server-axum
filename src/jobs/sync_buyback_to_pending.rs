use crate::{
    repositories::stocks::sync_buyback_periods_to_pending,
    state::AppState,
    structs::jobs::AppJob,
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct SyncBuybackToPendingJob;

#[async_trait]
impl AppJob for SyncBuybackToPendingJob {
    fn cron_expression(&self) -> &str {
        "0 10 8,20 * * *"
    }

    async fn run(&self, state: AppState) {
        match sync_buyback_periods_to_pending(&state).await {
            Ok(n) => tracing::info!("sync_buyback_to_pending inserted {} rows", n),
            Err(e) => tracing::error!("sync_buyback_to_pending fail: {}", e),
        }
    }
}
