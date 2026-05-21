use crate::{
    errors::{AppError, RequestError},
    repositories::images as images_repo,
    repositories::images::ImageRecord,
    state::AppState,
};
use axum::extract::Multipart;

pub async fn get_images(state: &AppState) -> Result<Vec<ImageRecord>, AppError> {
    images_repo::get_all_images(state).await
}

pub async fn cleanup_unused_images(state: &AppState) {
    let records = match images_repo::take_old_unused_images(state).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("cleanup_unused_images db error: {}", e);
            return;
        }
    };
    for r in &records {
        if let Err(e) = state.get_storage().delete(&r.storage_key).await {
            tracing::error!("cleanup_unused_images storage delete failed {}: {}", r.storage_key, e);
        }
    }
}

pub async fn delete_image(state: &AppState, id: i32) -> Result<(), AppError> {
    let storage_key = images_repo::delete_image_by_id(state, id).await?;
    if let Err(e) = state.get_storage().delete(&storage_key).await {
        tracing::error!("storage delete failed for key {}: {}", storage_key, e);
    }
    Ok(())
}

pub async fn upload_images(state: &AppState, mut multipart: Multipart) -> Result<Vec<ImageRecord>, AppError> {
    let mut records = vec![];

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
    {
        let content_type = field.content_type().unwrap_or("image/jpeg").to_string();
        let (storage_key, url) = state
            .get_storage()
            .upload(field, &content_type)
            .await
            .map_err(|e| RequestError::MultipartError(e.into()))?;
        let record = images_repo::insert_image(state, &storage_key, &url).await?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(RequestError::InvalidContent("no file provided".into()).into());
    }

    Ok(records)
}
