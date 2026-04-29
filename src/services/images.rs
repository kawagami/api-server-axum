use crate::{
    errors::{AppError, RequestError},
    repositories::images as images_repo,
    repositories::images::ImageRecord,
    state::AppStateV2,
};
use axum::body::Bytes;
use futures_util::Stream;

pub async fn get_images(state: &AppStateV2) -> Result<Vec<ImageRecord>, AppError> {
    images_repo::get_all_images(state).await
}

pub async fn cleanup_unused_images(state: &AppStateV2) {
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

pub async fn delete_image(state: &AppStateV2, id: i32) -> Result<(), AppError> {
    let storage_key = images_repo::delete_image_by_id(state, id).await?;
    if let Err(e) = state.get_storage().delete(&storage_key).await {
        tracing::error!("storage delete failed for key {}: {}", storage_key, e);
    }
    Ok(())
}

pub async fn upload_image<S, E>(
    state: &AppStateV2,
    stream: S,
    content_type: &str,
) -> Result<ImageRecord, AppError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<axum::BoxError>,
{
    let (storage_key, url) = state
        .get_storage()
        .upload(stream, content_type)
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?;

    images_repo::insert_image(state, &storage_key, &url).await
}
