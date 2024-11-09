use crate::{errors::AppError, state::AppStateV2};
use axum::{
    extract::{Multipart, State},
    Json,
};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct FirebaseImage {
    image_url: String,
}

pub async fn upload(
    State(state): State<AppStateV2>,
    mut multipart: Multipart,
) -> Result<Json<FirebaseImage>, AppError> {
    let client = Client::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::GetNextFieldFail)?
    {
        // let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.map_err(|_| AppError::ReadBytesFail)?;

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
            .map_err(|_| AppError::ConnectFail)?;

        // Check the FirebaseImage status and body if needed
        if res.status().is_success() {
            let upload_response = res
                .json::<FirebaseImage>()
                .await
                .map_err(|_| AppError::InvalidJson)?;

            // Retrieve the database pool from state
            let pool = state.get_pool().await;

            // Save image URL to the database and return the FirebaseImage
            let _: Result<FirebaseImage, _> = sqlx::query_as(
                r#"
                INSERT INTO firebase_images (image_url)
                VALUES ($1)
                RETURNING image_url
                "#,
            )
            .bind(&upload_response.image_url)
            .fetch_one(&pool)
            .await;

            // Return the inserted image data as JSON
            return Ok(Json(FirebaseImage {
                image_url: upload_response.image_url,
            }));
        }
    }

    Err(AppError::NotThing)
}
