use crate::{
    errors::AppError,
    state::AppStateV2,
    structs::firebase::{ApiResponse, DeleteImageRequest, FirebaseImage, Image},
};
use axum::{
    extract::{Multipart, State},
    Json,
};
use reqwest::multipart;

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
        // let name = field.name().unwrap().to_string();
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

        let res = client
            .post("http://fastapi-upload:8000/upload-image")
            // .post("http://host.docker.internal:8000/upload-image")
            .multipart(form)
            .send()
            .await
            .map_err(|err| {
                tracing::error!("HTTP request failed: {:?}", err);
                AppError::ConnectFail
            })?;

        // Check the FirebaseImage status and body if needed
        if res.status().is_success() {
            let upload_response = res
                .json::<FirebaseImage>()
                .await
                .map_err(|_| AppError::InvalidJson)?;

            // Save image URL to the database and return the FirebaseImage
            sqlx::query(
                r#"
                INSERT INTO firebase_images (image_url)
                VALUES ($1)
                "#,
            )
            .bind(&upload_response.image_url)
            .execute(&state.get_pool())
            .await
            .map_err(|err| {
                tracing::error!("Database insert failed: {:?}", err);
                AppError::DbInsertFail
            })?;

            // Return the inserted image data as JSON
            return Ok(Json(FirebaseImage {
                image_url: upload_response.image_url,
            }));
        }
    }

    Err(AppError::NotThing)
}

pub async fn images(State(state): State<AppStateV2>) -> Result<Json<Vec<Image>>, AppError> {
    let client = state.get_http_client();
    let api_url = "http://fastapi-upload:8000/list-images";
    // let api_url = "http://host.docker.internal:8000/list-images";

    let response = client
        .get(api_url)
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
    let api_url = "http://fastapi-upload:8000/delete-image";
    // let api_url = "http://host.docker.internal:8000/delete-image";
    let response = client
        .delete(api_url)
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
