use crate::{
    errors::AppError,
    repositories::images::ImageRecord,
    services::images as images_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(get_images))
            .route("/upload_multiple", post(upload_multiple))
            .route("/{id}", delete(delete_image)),
    )
}

async fn get_images(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ImageRecord>>, AppError> {
    auth_user.require_permission(Perm::ImageRead)?;
    Ok(Json(images_service::get_images(state.get_pool()).await?))
}

async fn delete_image(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::ImageDelete)?;
    images_service::delete_image(state.get_pool(), state.get_storage(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upload_multiple(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth_user.require_permission(Perm::ImageCreate)?;
    let base_url = state
        .get_settings()
        .get("upload_base_url")
        .unwrap_or_else(|| "https://axum.kawa.homes/uploads".to_string());
    let records = images_service::upload_images(state.get_pool(), state.get_storage(), &base_url, multipart).await?;
    let body: Vec<_> = records
        .iter()
        .map(|r| serde_json::json!({ "id": r.id, "url": r.url }))
        .collect();
    Ok((StatusCode::CREATED, Json(serde_json::json!(body))))
}
