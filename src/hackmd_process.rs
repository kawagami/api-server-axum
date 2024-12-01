use axum::{extract::State, http::StatusCode, response::IntoResponse};
use reqwest::Client;
// use serde_json::Value;
use std::env;

use crate::{state::AppStateV2, structs::hackmd::Post};

pub async fn _fetch_notes_handler(State(state): State<AppStateV2>) -> impl IntoResponse {
    // 取得環境變數中的 Token
    let token = env::var("HACKMD_TOKEN").expect("HACKMD_TOKEN not set");

    // 構建請求 URL
    let url = "https://api.hackmd.io/v1/notes";

    // 使用 reqwest 發送請求
    let client = Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status() == StatusCode::OK {
                // 解析回應為 JSON 並轉換為 `Vec<Post>` (假設 API 回應是一個 Post 的陣列)
                match resp.json::<Vec<Post>>().await {
                    Ok(posts) => {
                        let _ = state.insert_posts_handler(posts).await;

                        (StatusCode::OK, "success".to_string())
                    }
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

pub async fn fetch_notes_job(state: AppStateV2) -> impl IntoResponse {
    // 取得環境變數中的 Token
    let token = env::var("HACKMD_TOKEN").expect("HACKMD_TOKEN not set");

    // 構建請求 URL
    let url = "https://api.hackmd.io/v1/notes";

    // 使用 reqwest 發送請求
    let client = Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status() == StatusCode::OK {
                // 解析回應為 JSON 並轉換為 `Vec<Post>` (假設 API 回應是一個 Post 的陣列)
                match resp.json::<Vec<Post>>().await {
                    Ok(posts) => {
                        let _ = state.insert_posts_handler(posts).await;
                        tracing::info!("insert_posts_handler success");

                        (StatusCode::OK, "success".to_string())
                    }
                    Err(_) => {
                        tracing::error!("Failed to parse response");

                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to parse response".to_string(),
                        )
                    }
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
