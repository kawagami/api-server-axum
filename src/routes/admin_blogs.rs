use axum::{
    extract::{Extension, Path, State},
    routing::put,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    services::blogs as blogs_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        blogs::PutBlog,
        roles::Perm,
        ws::WsEvent,
    },
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new().route("/{id}", put(put_blog).delete(delete_blog)),
    )
}

async fn put_blog(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(blog): Json<PutBlog>,
) -> Result<Json<()>, AppError> {
    auth_user.require_permission(Perm::BlogUpdate)?;
    let title = blogs_service::upsert_blog(&state, id, blog).await?;
    state.broadcast(WsEvent::BlogCreated, serde_json::json!({ "id": id, "title": title }));
    Ok(Json(()))
}

async fn delete_blog(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    auth_user.require_permission(Perm::BlogDelete)?;
    blogs_service::delete_blog_with_images(&state, id).await?;
    Ok(Json(()))
}
