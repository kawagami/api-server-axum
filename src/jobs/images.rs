use crate::{services::images as images_service, state::AppStateV2, structs::jobs::AppJob};
use async_trait::async_trait;

#[derive(Clone)]
pub struct CleanupUnusedImagesJob;

#[async_trait]
impl AppJob for CleanupUnusedImagesJob {
    fn cron_expression(&self) -> &str {
        "0 0 * * * *" // 每小時執行一次
    }

    async fn run(&self, state: AppStateV2) {
        images_service::cleanup_unused_images(&state).await;
    }
}
