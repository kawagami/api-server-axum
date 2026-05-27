use crate::{services::stocks::stock_day_all_service, state::AppState, structs::jobs::AppJob};
use async_trait::async_trait;

pub struct FetchStockDayAllJob;

#[async_trait]
impl AppJob for FetchStockDayAllJob {
    fn cron_expression(&self) -> &str {
        "0 0 20 * * *"
    }

    async fn run(&self, state: AppState) {
        super::run_with_retries(
            "stock_day_all_service",
            3,
            std::time::Duration::from_secs(3600),
            || stock_day_all_service(&state),
        )
        .await;
    }
}
