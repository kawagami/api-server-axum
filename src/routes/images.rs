use crate::{
    errors::AppError,
    middleware::auth,
    repositories::images::ImageRecord,
    services::images as images_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_images))
        .route("/upload", post(upload_image))
        .route("/{id}", delete(delete_image))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ))
}

async fn get_images(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ImageRecord>>, AppError> {
    auth_user.require_permission(Perm::ImageRead)?;
    Ok(Json(images_service::get_images(&state).await?))
}

async fn delete_image(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::ImageDelete)?;
    images_service::delete_image(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upload_image(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth_user.require_permission(Perm::ImageWrite)?;
    let record = images_service::upload_image(&state, multipart).await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": record.id, "url": record.url })),
    ))
}
