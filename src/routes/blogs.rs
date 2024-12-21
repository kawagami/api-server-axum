use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    state::AppStateV2,
    structs::blogs::{Pagination, PutBlog},
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/:id", get(get_blog).delete(delete_blog).put(put_blog))
}

/// 取 blogs 清單
async fn get_blogs(
    Query(query): Query<Pagination>,
    State(state): State<AppStateV2>,
) -> impl IntoResponse {
    let offset = (query.page.saturating_sub(1)) * query.per_page;

    match state
        .get_blogs_with_pagination(query.per_page, offset)
        .await
    {
        Ok(result) => Json(result).into_response(), // 使用 `.into_response()` 統一類型
        Err(err) => {
            // 將錯誤回應轉換為 Response 類型
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", err)).into_response()
        }
    }
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

async fn delete_blog(State(state): State<AppStateV2>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let result = state.delete_blog(id).await;

    tracing::debug!("delete_blog result => {:?}", result);
    Json(format!("delete_blog 收到 id => {}", id))
}

async fn put_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
    Json(blog): Json<PutBlog>,
) -> impl IntoResponse {
    let tocs = blog.extract_toc_texts();
    let result = state
        .upsert_blog(id, blog.markdown, blog.html, tocs, blog.tags)
        .await;

    tracing::debug!("put_blog result => {:?}", result);
    Json(format!("put_blog 收到\nid => {}\n", id))
}
