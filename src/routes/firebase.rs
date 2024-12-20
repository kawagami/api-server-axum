use crate::{
    auth,
    errors::AppError,
    state::AppStateV2,
    structs::firebase::{ApiResponse, DeleteImageRequest, FirebaseImage, Image},
};
use axum::{
    extract::{Multipart, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use reqwest::multipart;

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    let router = Router::new().route("/", get(images));
    let middleware_router = Router::new()
        .route("/", post(upload).delete(delete))
        .layer(middleware::from_fn_with_state(state, auth::authorize));
    router.merge(middleware_router)
}

pub async fn upload(
    State(state): State<AppStateV2>,
    mut multipart: Multipart,
) -> Result<Json<FirebaseImage>, AppError> {
    let client = state.get_http_client();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::GetNextFieldFail)?
    {
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.map_err(|err| {
            tracing::error!("Failed to read bytes: {:?}", err);
            AppError::ReadBytesFail
        })?;

        // Create a form part with the received file
        let part = multipart::Part::bytes(data.to_vec())
            .file_name(file_name.clone())
            .mime_str(&content_type)
            .unwrap();

        // Create a multipart form
        let form = multipart::Form::new().part("file", part);

        let url = format!("{}{}", state.get_fastapi_upload_host(), "/upload-image");

        let res = client
            .post(url)
            .multipart(form)
            .send()
            .await
            .map_err(|err| {
                tracing::error!("HTTP request failed: {:?}", err);
                AppError::ConnectFail
            })?;

        // Check the response status and parse the JSON
        if res.status().is_success() {
            let upload_response: FirebaseImage =
                res.json().await.map_err(|_| AppError::InvalidJson)?;

            // Return the inserted image data as JSON
            return Ok(Json(upload_response));
        }
    }

    Err(AppError::NotThing)
}

pub async fn images(State(state): State<AppStateV2>) -> Result<Json<Vec<Image>>, AppError> {
    let client = state.get_http_client();

    let url = format!("{}{}", state.get_fastapi_upload_host(), "/list-images");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| {
            tracing::error!("HTTP request failed: {:?}", err);
            AppError::ConnectFail
        })?
        .json::<ApiResponse>()
        .await
        .map_err(|err| {
            tracing::error!("Failed to parse response JSON: {:?}", err);
            AppError::InvalidResponse
        })?;

    Ok(Json(response.files))
}

pub async fn delete(
    State(state): State<AppStateV2>,
    Json(delete_data): Json<DeleteImageRequest>,
) -> Result<Json<()>, AppError> {
    let client = state.get_http_client();

    let url = format!("{}{}", state.get_fastapi_upload_host(), "/delete-image");

    let response = client
        .delete(url)
        .json(&delete_data)
        .send()
        .await
        .map_err(|err| {
            tracing::error!("HTTP request failed: {:?}", err);
            AppError::ConnectFail
        })?;

    tracing::debug!("{:?}", response);
    Ok(Json(()))
}
