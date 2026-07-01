use crate::{repositories::notes, state::AppState, structs::notes::Post};

pub async fn run(state: AppState) {
    let token = match state.get_settings().get("hackmd_token") {
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
    if let Err(err) = notes::insert_posts_handler(state.get_pool(), posts).await {
        tracing::error!(job = "FetchNotesJob", "notes sync failed: {}", err);
        return;
    }

    tracing::info!(job = "FetchNotesJob", "synced {} notes", count);
}
