use crate::{
    state::AppStateV2,
    structs::{hackmd::Post, jobs::AppJob},
};
use async_trait::async_trait;
use axum::http::StatusCode;
use std::env;

#[derive(Clone)]
pub struct FetchNotesJob;

#[async_trait]
impl AppJob for FetchNotesJob {
    fn cron_expression(&self) -> &str {
        "0 0 * * * *" // 每小時執行一次
    }

    async fn run(&self, state: AppStateV2) {
        let token = match env::var("HACKMD_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                tracing::error!("HACKMD_TOKEN not set");
                return;
            }
        };

        let url = "https://api.hackmd.io/v1/notes";
        let client = state.get_http_client();

        match client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    match resp.json::<Vec<Post>>().await {
                        Ok(posts) => {
                            if let Err(err) = state.insert_posts_handler(posts).await {
                                tracing::error!("insert_posts_handler failed: {}", err);
                            } else {
                                tracing::info!("insert_posts_handler success");
                            }
                        }
                        Err(err) => {
                            tracing::error!("Failed to parse response: {}", err);
                        }
                    }
                } else {
                    tracing::error!("Unexpected status code: {}", resp.status());
                }
            }
            Err(err) => {
                tracing::error!("Request error: {}", err);
            }
        }
    }
}
