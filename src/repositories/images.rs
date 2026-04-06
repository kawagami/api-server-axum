use crate::{errors::AppError, state::AppStateV2};
use serde::Serialize;
use sqlx::Row;

#[derive(Serialize)]
pub struct ImageRecord {
    pub id: i32,
    pub storage_key: String,
    pub url: String,
}

pub async fn get_all_images(state: &AppStateV2) -> Result<Vec<ImageRecord>, AppError> {
    let rows = sqlx::query("SELECT id, storage_key, url FROM images ORDER BY id DESC")
        .fetch_all(state.get_pool())
        .await?;

    Ok(rows
        .iter()
        .map(|row| ImageRecord {
            id: row.get("id"),
            storage_key: row.get("storage_key"),
            url: row.get("url"),
        })
        .collect())
}

pub async fn insert_image(
    state: &AppStateV2,
    storage_key: &str,
    url: &str,
) -> Result<ImageRecord, AppError> {
    let row = sqlx::query(
        "INSERT INTO images (storage_key, url) VALUES ($1, $2) RETURNING id, storage_key, url",
    )
    .bind(storage_key)
    .bind(url)
    .fetch_one(state.get_pool())
    .await?;

    Ok(ImageRecord {
        id: row.get("id"),
        storage_key: row.get("storage_key"),
        url: row.get("url"),
    })
}

pub async fn _delete_image_by_key(state: &AppStateV2, storage_key: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM images WHERE storage_key = $1")
        .bind(storage_key)
        .execute(state.get_pool())
        .await?;

    Ok(())
}
