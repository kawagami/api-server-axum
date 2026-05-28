use crate::{
    repositories::notes,
    state::AppState,
    structs::{jobs::AppJob, notes::Post},
};
use async_trait::async_trait;

pub struct FetchNotesJob;

#[async_trait]
impl AppJob for FetchNotesJob {
    fn cron_expression(&self) -> &str {
        "0 0 19 * * *" // 每日 UTC 19:00（UTC+8 03:00）
    }

    async fn run(&self, state: AppState) {
        let token = match state.get_setting("hackmd_token") {
            Some(t) if !t.is_empty() => t,
            _ => return,
        };

        const HACKMD_URL: &str = "https://api.hackmd.io/v1/notes";

        let client = state.get_http_client();
        let response = match client
            .get(HACKMD_URL)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                tracing::error!(job = "FetchNotesJob", "HackMD request failed: {}", err);
                return;
            }
        };

        if !response.status().is_success() {
            tracing::error!(job = "FetchNotesJob", "HackMD returned {}", response.status());
            return;
        }

        let posts: Vec<Post> = match response.json().await {
            Ok(posts) => posts,
            Err(err) => {
                tracing::error!(job = "FetchNotesJob", "HackMD response parse failed: {}", err);
                return;
            }
        };

        let count = posts.len();
        if let Err(err) = notes::insert_posts_handler(&state, posts).await {
            tracing::error!(job = "FetchNotesJob", "notes sync failed: {}", err);
            return;
        }

        tracing::info!(job = "FetchNotesJob", "synced {} notes", count);
    }
}
