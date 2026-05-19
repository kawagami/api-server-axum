use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    services::blogs as blogs_service,
    state::AppState,
    structs::blogs::{BlogsResponse, DbBlog, Pagination},
};

pub fn new() -> Router<AppState> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/tags", get(get_tags))
        .route("/{id}", get(get_blog))
}

async fn get_blogs(
    Query(query): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<BlogsResponse>, AppError> {
    let blogs = blogs_service::get_blogs(&state, query.page, query.per_page, query.tag).await?;
    Ok(Json(blogs))
}

async fn get_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, AppError> {
    let tags = blogs_service::get_tags(&state).await?;
    Ok(Json(tags))
}

async fn get_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DbBlog>, AppError> {
    let blog = blogs_service::get_blog(&state, id).await?;
    Ok(Json(blog))
}
