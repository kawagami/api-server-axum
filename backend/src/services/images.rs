use crate::{
    errors::{AppError, RequestError},
    repositories::images as images_repo,
    repositories::images::ImageRecord,
    storage::Storage,
};
use axum::extract::Multipart;
use sqlx::{Pool, Postgres};

pub async fn get_images(pool: &Pool<Postgres>) -> Result<Vec<ImageRecord>, AppError> {
    images_repo::get_all_images(pool).await
}

pub async fn cleanup_unused_images(pool: &Pool<Postgres>, storage: &Storage) {
    let records = match images_repo::take_old_unused_images(pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("cleanup_unused_images db error: {}", e);
            return;
        }
    };
    for r in &records {
        if let Err(e) = storage.delete(&r.storage_key).await {
            tracing::error!("cleanup_unused_images storage delete failed {}: {}", r.storage_key, e);
        }
    }
}

pub async fn delete_image(pool: &Pool<Postgres>, storage: &Storage, id: i32) -> Result<(), AppError> {
    let storage_key = images_repo::delete_image_by_id(pool, id).await?;
    if let Err(e) = storage.delete(&storage_key).await {
        tracing::error!("storage delete failed for key {}: {}", storage_key, e);
    }
    Ok(())
}

pub async fn upload_images(
    pool: &Pool<Postgres>,
    storage: &Storage,
    base_url: &str,
    mut multipart: Multipart,
) -> Result<Vec<ImageRecord>, AppError> {
    let mut records = vec![];

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
    {
        let content_type = field.content_type().unwrap_or("image/jpeg").to_string();
        let (storage_key, url) = storage
            .upload(field, &content_type, base_url)
            .await
            .map_err(|e| RequestError::MultipartError(e.into()))?;
        let record = images_repo::insert_image(pool, &storage_key, &url).await?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(RequestError::InvalidContent("no file provided".into()).into());
    }

    Ok(records)
}
