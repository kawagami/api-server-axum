use crate::{services::stocks::stock_day_all_service, state::AppState, structs::jobs::AppJob};
use async_trait::async_trait;

pub struct FetchStockDayAllJob;

#[async_trait]
impl AppJob for FetchStockDayAllJob {
    fn cron_expression(&self) -> &str {
        "0 0 20 * * *"
    }

    async fn run(&self, state: AppState) {
        const MAX_ATTEMPTS: u32 = 3;

        for attempt in 1..=MAX_ATTEMPTS {
            match stock_day_all_service(&state).await {
                Ok(_) => {
                    tracing::info!("stock_day_all_service success");
                    return;
                }
                Err(e) => {
                    if attempt < MAX_ATTEMPTS {
                        tracing::warn!(
                            "stock_day_all_service fail (attempt {}/{}): {}, retry in 1h",
                            attempt,
                            MAX_ATTEMPTS,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                    } else {
                        tracing::error!(
                            "stock_day_all_service fail (attempt {}/{}): {}",
                            attempt,
                            MAX_ATTEMPTS,
                            e
                        );
                    }
                }
            }
        }
    }
}
