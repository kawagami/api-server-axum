use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use serde_json::Value;

use crate::state::AppStateV2;

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_blogs).post(create_blog))
        .route("/:id", get(get_blog).delete(delete_blog).put(put_blog))
}

/// 取 blogs 清單
async fn get_blogs() -> impl IntoResponse {
    Json("這是 fn get_blogs")
}

/// 取 blog 詳細內容
async fn get_blog(Path(id): Path<String>) -> impl IntoResponse {
    // Json(format!("收到 id => {}", id))
    format!("get_blog 收到 id => {}", id)
}

async fn create_blog(Json(value): Json<Value>) -> impl IntoResponse {
    Json(value)
}

async fn delete_blog(Path(id): Path<String>) -> impl IntoResponse {
    format!("delete_blog 收到 id => {}", id)
}

async fn put_blog(Path(id): Path<String>, Json(value): Json<Value>) -> impl IntoResponse {
    format!("put_blog 收到\nid => {}\nvalue => {}\n", id, value)
}
