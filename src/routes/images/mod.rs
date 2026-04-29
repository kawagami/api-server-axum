use crate::{
    errors::{AppError, RequestError},
    middleware::auth,
    repositories::images::ImageRecord,
    services::images as images_service,
    state::AppStateV2,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_images))
        .route("/upload", post(upload_image))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

async fn get_images(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<ImageRecord>>, AppError> {
    Ok(Json(images_service::get_images(&state).await?))
}

async fn upload_image(
    State(state): State<AppStateV2>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
        .ok_or(RequestError::InvalidContent("no file provided".into()))?;

    let content_type = field.content_type().unwrap_or("image/jpeg").to_string();

    let record = images_service::upload_image(&state, field, &content_type).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": record.id, "url": record.url })),
    ))
}
