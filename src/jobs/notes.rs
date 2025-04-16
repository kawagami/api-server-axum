use crate::{
    repositories::notes,
    state::AppStateV2,
    structs::{jobs::AppJob, notes::Post},
};
use async_trait::async_trait;
use axum::http::StatusCode;
use std::env;

#[derive(Clone)]
pub struct FetchNotesJob;

#[async_trait]
impl AppJob for FetchNotesJob {
    fn enabled(&self) -> bool {
        std::env::var("ENABLE_FETCH_NOTES_JOB").unwrap_or_else(|_| "true".to_string()) == "true"
    }

    fn cron_expression(&self) -> &str {
        "0 0 * * * *" // 每小時執行一次
    }

    async fn run(&self, state: AppStateV2) {
        // 取得 HACKMD_TOKEN
        let token = env::var("HACKMD_TOKEN").unwrap_or_else(|_| {
            tracing::error!("HACKMD_TOKEN not set");
            return String::new();
        });
        if token.is_empty() {
            return;
        }

        // API URL
        const HACKMD_URL: &str = "https://api.hackmd.io/v1/notes";

        // 建立 HTTP 請求
        let client = state.get_http_client();
        let response = client
            .get(HACKMD_URL)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        let response = match response {
            Ok(resp) => resp,
            Err(err) => {
                tracing::error!("Request error: {}", err);
                return;
            }
        };

        // 確認 HTTP 狀態碼
        if response.status() != StatusCode::OK {
            tracing::error!("Unexpected status code: {}", response.status());
            return;
        }

        // 解析 JSON
        let posts: Vec<Post> = match response.json().await {
            Ok(posts) => posts,
            Err(err) => {
                tracing::error!("Failed to parse response: {}", err);
                return;
            }
        };

        // 插入資料
        if let Err(err) = notes::insert_posts_handler(&state, posts).await {
            tracing::error!("insert_posts_handler failed: {}", err);
            return;
        }

        tracing::info!("insert_posts_handler success");
    }
}
