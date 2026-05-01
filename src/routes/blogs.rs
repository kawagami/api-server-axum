use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    services::blogs as blogs_service,
    state::AppStateV2,
    structs::blogs::{DbBlog, Pagination, PutBlog},
    structs::ws::WsEvent,
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_blogs))
        .route("/{id}", get(get_blog).delete(delete_blog).put(put_blog))
}

async fn get_blogs(
    Query(query): Query<Pagination>,
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<DbBlog>>, AppError> {
    let blogs = blogs_service::get_blogs(&state, query.page, query.per_page).await?;
    Ok(Json(blogs))
}

async fn get_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
) -> Result<Json<DbBlog>, AppError> {
    let blog = blogs_service::get_blog(&state, id).await?;
    Ok(Json(blog))
}

async fn delete_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    blogs_service::delete_blog_with_images(&state, id).await?;
    Ok(Json(()))
}

async fn put_blog(
    State(state): State<AppStateV2>,
    Path(id): Path<Uuid>,
    Json(blog): Json<PutBlog>,
) -> Result<Json<()>, AppError> {
    let title = blog.extract_toc_texts().into_iter().next().unwrap_or_default();
    blogs_service::upsert_blog(&state, id, blog).await?;
    state.broadcast(WsEvent::BlogCreated, serde_json::json!({ "id": id, "title": title }));
    Ok(Json(()))
}
