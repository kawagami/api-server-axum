use crate::{state::AppStateV2, structs::jobs::AppJob};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ExampleJob;

#[async_trait]
impl AppJob for ExampleJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *" // 每分鐘執行一次
    }

    async fn run(&self, _state: AppStateV2) {
        // 要執行的邏輯
    }
}
