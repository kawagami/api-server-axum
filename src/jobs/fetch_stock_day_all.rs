use crate::{services::stocks::stock_day_all_service, state::AppState, structs::jobs::AppJob};
use async_trait::async_trait;

#[derive(Clone)]
pub struct FetchStockDayAllJob;

#[async_trait]
impl AppJob for FetchStockDayAllJob {
    fn cron_expression(&self) -> &str {
        "0 0 20 * * *"
    }

    async fn run(&self, state: AppState) {
        match stock_day_all_service(&state).await {
            Ok(_) => tracing::info!("stock_day_all_service success"),
            Err(e) => tracing::error!("stock_day_all_service fail: {}", e),
        }
    }
}
