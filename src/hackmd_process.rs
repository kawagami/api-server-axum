use axum::{http::StatusCode, response::IntoResponse};
use reqwest::Client;
use serde_json::Value;
use std::env;

pub async fn _fetch_notes_handler() -> impl IntoResponse {
    // 取得環境變數中的 Token
    let token = env::var("HACKMD_TOKEN").expect("HACKMD_TOKEN not set");

    // 構建請求 URL
    let url = format!("{}", "https://api.hackmd.io/v1/notes");

    // 使用 reqwest 發送請求
    let client = Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status() == StatusCode::OK {
                // 解析回應為 JSON
                match resp.json::<Value>().await {
                    Ok(note) => (StatusCode::OK, serde_json::to_string(&note).unwrap()),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to parse response".to_string(),
                    ),
                }
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Unexpected status code: {}", resp.status()),
                )
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Request error: {}", err),
        ),
    }
}
