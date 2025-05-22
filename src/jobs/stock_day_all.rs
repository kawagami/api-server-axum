use crate::{services::stocks::stock_day_all_service, state::AppStateV2, structs::jobs::AppJob};
use async_trait::async_trait;

#[derive(Clone)]
pub struct StockDayAllJob;

#[async_trait]
impl AppJob for StockDayAllJob {
    fn cron_expression(&self) -> &str {
        "0 0 0,8,16 * * *" // 特定三個時間點執行
    }

    async fn run(&self, state: AppStateV2) {
        // 每天抓一次 stock day all 的 API 資料進資料庫
        match stock_day_all_service(&state).await {
            Ok(_) => tracing::info!("job 抓 stock day all 的 API 資料進資料庫成功"),
            Err(e) => tracing::error!("job stock_day_all_service fail: {}", e),
        }
    }
}
