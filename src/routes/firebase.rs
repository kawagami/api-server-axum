use crate::{
    errors::AppError,
    state::AppStateV2,
    structs::firebase::{DbFirebaseImage, FirebaseImage, ResponseFirebaseImage},
};
use axum::{
    extract::{Multipart, State},
    Json,
};
use chrono::FixedOffset;
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

impl DbFirebaseImage {
    fn to_local_formatted(&self) -> ResponseFirebaseImage {
        // 設定 UTC+8 的時區偏移
        let utc_plus_8 = FixedOffset::east_opt(8 * 3600).expect("east_opt fail");

        // 轉換 `created_at` 和 `updated_at` 到當前時區並格式化
        let created_at = self
            .created_at
            .with_timezone(&utc_plus_8)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let updated_at = self
            .updated_at
            .with_timezone(&utc_plus_8)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        ResponseFirebaseImage {
            id: self.id,
            image_url: self.image_url.clone(),
            created_at,
            updated_at,
        }
    }
}

pub async fn images(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<ResponseFirebaseImage>>, AppError> {
    let pool = state.get_pool();

    let images: Vec<DbFirebaseImage> = sqlx::query_as(
        r#"
            SELECT
                    id,
                    image_url,
                    created_at,
                    updated_at
            FROM
                    firebase_images
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|err| {
        tracing::debug!("{}", err);
        AppError::GetDbDataFail
    })?;

    // 使用 iter().map() 來簡化 for 迴圈
    let response: Vec<ResponseFirebaseImage> = images
        .iter()
        .map(|image| image.to_local_formatted())
        .collect();

    Ok(Json(response))
}
