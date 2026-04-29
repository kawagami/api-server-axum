use crate::{errors::{AppError, RequestError}, state::AppStateV2};
use serde::Serialize;
use sqlx::{PgConnection, Row};

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

pub async fn get_images_by_urls(
    state: &AppStateV2,
    urls: &[String],
) -> Result<Vec<ImageRecord>, AppError> {
    let rows = sqlx::query("SELECT id, storage_key, url FROM images WHERE url = ANY($1)")
        .bind(urls)
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

pub async fn delete_image_by_id(state: &AppStateV2, id: i32) -> Result<String, AppError> {
    let row = sqlx::query("DELETE FROM images WHERE id = $1 RETURNING storage_key")
        .bind(id)
        .fetch_optional(state.get_pool())
        .await?
        .ok_or(RequestError::NotFound)?;
    Ok(row.get("storage_key"))
}

pub async fn delete_images_in_tx(
    conn: &mut PgConnection,
    ids: &[i32],
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM images WHERE id = ANY($1)")
        .bind(ids)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
