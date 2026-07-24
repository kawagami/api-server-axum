use crate::{
    errors::AppError,
    repositories::images::{self as images_repo, ImageRecord},
    services::images as images_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    routing::{delete, get},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(get_images).post(upload_image))
            .route("/{id}", delete(delete_image)),
    )
}

async fn get_images(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ImageRecord>>, AppError> {
    auth_user.require_permission(Perm::ImageRead)?;
    Ok(Json(images_service::get_images(state.get_pool(), auth_user.owner_filter()).await?))
}

async fn delete_image(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::ImageDelete)?;
    auth_user.require_owner(images_repo::get_owner(state.get_pool(), id).await?)?;
    images_service::delete_image(state.get_pool(), state.get_storage(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upload_image(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    auth_user.require_permission(Perm::ImageCreate)?;
    let settings = state.get_settings();
    let base_url = settings
        .get("upload_base_url")
        .unwrap_or_else(|| "https://media.kawa.homes".to_string());
    // PATCH 端已驗證 1–100；此處仍給 fallback，避免舊資料/缺 key 時炸掉
    let quality = settings
        .get("image_webp_quality")
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(images_service::DEFAULT_WEBP_QUALITY);
    let record = images_service::upload_image(state.get_pool(), state.get_storage(), &base_url, Some(auth_user.id), quality, multipart).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": record.id, "url": record.url }))))
}
