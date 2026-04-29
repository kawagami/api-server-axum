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
