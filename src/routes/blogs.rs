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
    structs::blogs::{DbBlog, Pagination},
};

pub fn new() -> Router<AppState> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/{id}", get(get_blog))
}

async fn get_blogs(
    Query(query): Query<Pagination>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DbBlog>>, AppError> {
    let blogs = blogs_service::get_blogs(&state, query.page, query.per_page).await?;
    Ok(Json(blogs))
}

async fn get_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DbBlog>, AppError> {
    let blog = blogs_service::get_blog(&state, id).await?;
    Ok(Json(blog))
}
