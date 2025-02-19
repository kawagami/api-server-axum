use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    repositories::blogs,
    state::AppStateV2,
    structs::blogs::{DbBlog, Pagination, PutBlog},
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/{id}", get(get_blog).delete(delete_blog).put(put_blog))
}

/// 取 blogs 清單
async fn get_blogs(
    Query(query): Query<Pagination>,
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<DbBlog>>, AppError> {
    let offset = (query.page.saturating_sub(1)) * query.per_page;

    let blogs = blogs::get_blogs_with_pagination(&state, query.per_page, offset).await?;

    Ok(Json(blogs))
}

/// 取 blog 詳細內容
async fn get_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
) -> Result<Json<DbBlog>, AppError> {
    let blog = blogs::get_blog_by_id(&state, id).await?;

    Ok(Json(blog))
}

async fn delete_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    let result = blogs::delete_blog(&state, id).await?;

    Ok(Json(result))
}

async fn put_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
    Json(blog): Json<PutBlog>,
) -> Result<Json<()>, AppError> {
    let tocs = blog.extract_toc_texts();
    let result = blogs::upsert_blog(&state, id, blog.markdown, tocs, blog.tags).await?;

    Ok(Json(result))
}
