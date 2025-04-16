use crate::state::AppStateV2;
use async_trait::async_trait;

#[async_trait]
pub trait AppJob {
    // 取得 tokio-cron-scheduler 格式的執行時間
    fn cron_expression(&self) -> &str;

    // 要執行的任務
    async fn run(&self, state: AppStateV2);

    fn enabled(&self) -> bool {
        true
    }
}
