use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    state::AppStateV2,
    structs::blogs::{CreateBlog, UpdateBlog},
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_blogs).post(create_blog))
        .route("/:id", get(get_blog).delete(delete_blog).put(put_blog))
}

/// 取 blogs 清單
async fn get_blogs(State(state): State<AppStateV2>) -> impl IntoResponse {
    let result = state.get_all_blogs().await.expect("get_blogs fail");
    Json(result)
}

/// 取 blog 詳細內容
async fn get_blog(State(state): State<AppStateV2>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.get_blog_by_id(id).await {
        Ok(blog) => (StatusCode::OK, Json(blog)).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!([])), // 使用空陣列作為錯誤返回
        )
            .into_response(),
    }
}

async fn create_blog(
    State(state): State<AppStateV2>,
    Json(blog): Json<CreateBlog>,
) -> impl IntoResponse {
    let result = state
        .insert_blog(&blog.id, &blog.markdown, &blog.html, &blog.tags)
        .await;

    tracing::debug!("create_blog result => {:?}", result);
    Json(blog)
}

async fn delete_blog(State(state): State<AppStateV2>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let result = state.delete_blog(id).await;

    tracing::debug!("delete_blog result => {:?}", result);
    Json(format!("delete_blog 收到 id => {}", id))
}

async fn put_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
    Json(blog): Json<UpdateBlog>,
) -> impl IntoResponse {
    let markdown = blog.markdown;
    let html = blog.html;
    let tags = blog.tags;

    let result = state.update_blog(id, markdown, html, tags).await;

    tracing::debug!("put_blog result => {:?}", result);
    Json(format!("put_blog 收到\nid => {}\n", id))
}
