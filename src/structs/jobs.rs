use crate::state::AppState;
use async_trait::async_trait;

#[async_trait]
pub trait AppJob {
    fn cron_expression(&self) -> &str;
    async fn run(&self, state: AppState);

    fn enabled(&self, _state: &AppState) -> bool {
        true
    }
}
