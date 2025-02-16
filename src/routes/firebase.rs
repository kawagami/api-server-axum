use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::firebase::{delete as repo_delete, images as repo_images, upload as repo_upload},
    routes::auth,
    state::AppStateV2,
    structs::firebase::{DeleteImageRequest, FirebaseImage, Image},
};
use axum::{
    extract::{Multipart, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use reqwest::multipart;

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new().route("/", get(images)).nest(
        "/",
        Router::new()
            .route("/", post(upload).delete(delete))
            .layer(middleware::from_fn_with_state(state, auth::authorize)),
    )
}

pub async fn upload(
    State(state): State<AppStateV2>,
    mut multipart: Multipart,
) -> Result<Json<FirebaseImage>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::RequestError(RequestError::MultipartError(err.into())))?
    {
        let file_name = field
            .file_name()
            .ok_or_else(|| {
                AppError::RequestError(RequestError::InvalidContent(
                    "Missing file name".to_string(),
                ))
            })?
            .to_string();

        let content_type = field
            .content_type()
            .ok_or_else(|| {
                AppError::RequestError(RequestError::InvalidContent(
                    "Missing content type".to_string(),
                ))
            })?
            .to_string();

        let data = field
            .bytes()
            .await
            .map_err(|err| AppError::RequestError(RequestError::MultipartError(err.into())))?;

        let part = multipart::Part::bytes(data.to_vec())
            .file_name(file_name.clone())
            .mime_str(&content_type)
            .map_err(|err| AppError::RequestError(RequestError::InvalidContent(err.to_string())))?;

        let form = multipart::Form::new().part("file", part);
        let res = repo_upload(&state, form).await?;

        if res.status().is_success() {
            return res
                .json()
                .await
                .map(Json)
                .map_err(|err| AppError::RequestError(RequestError::InvalidJson(err.into())));
        }
    }

    Err(AppError::RequestError(RequestError::NotFound))
}

pub async fn images(State(state): State<AppStateV2>) -> Result<Json<Vec<Image>>, AppError> {
    let images = repo_images(&state)
        .await
        .map_err(|err| {
            tracing::error!("Failed to fetch images: {}", err);
            AppError::SystemError(SystemError::Internal("Failed to fetch images".to_string()))
        })
        .unwrap_or_default();

    Ok(Json(images))
}

pub async fn delete(
    State(state): State<AppStateV2>,
    Json(delete_data): Json<DeleteImageRequest>,
) -> Result<Json<()>, AppError> {
    repo_delete(&state, delete_data).await.map_err(|err| {
        tracing::error!("Failed to delete image: {:?}", err);
        AppError::SystemError(SystemError::Internal("Failed to delete image".to_string()))
    })?;

    Ok(Json(()))
}
