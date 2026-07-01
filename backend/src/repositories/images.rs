use crate::errors::{AppError, RequestError};
use serde::Serialize;
use sqlx::{PgConnection, Pool, Postgres, Row};

#[derive(Serialize)]
pub struct ImageRecord {
    pub id: i32,
    pub storage_key: String,
    pub url: String,
    pub status: String,
}

pub async fn get_all_images(pool: &Pool<Postgres>) -> Result<Vec<ImageRecord>, AppError> {
    let rows = sqlx::query("SELECT id, storage_key, url, status FROM images ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    Ok(rows
        .iter()
        .map(|row| ImageRecord {
            id: row.get("id"),
            storage_key: row.get("storage_key"),
            url: row.get("url"),
            status: row.get("status"),
        })
        .collect())
}

pub async fn insert_image(
    pool: &Pool<Postgres>,
    storage_key: &str,
    url: &str,
) -> Result<ImageRecord, AppError> {
    let row = sqlx::query(
        "INSERT INTO images (storage_key, url) VALUES ($1, $2) RETURNING id, storage_key, url, status",
    )
    .bind(storage_key)
    .bind(url)
    .fetch_one(pool)
    .await?;

    Ok(ImageRecord {
        id: row.get("id"),
        storage_key: row.get("storage_key"),
        url: row.get("url"),
        status: row.get("status"),
    })
}

pub async fn get_images_by_urls(
    pool: &Pool<Postgres>,
    urls: &[String],
) -> Result<Vec<ImageRecord>, AppError> {
    let rows = sqlx::query("SELECT id, storage_key, url, status FROM images WHERE url = ANY($1)")
        .bind(urls)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .iter()
        .map(|row| ImageRecord {
            id: row.get("id"),
            storage_key: row.get("storage_key"),
            url: row.get("url"),
            status: row.get("status"),
        })
        .collect())
}

pub async fn mark_images_active_by_urls_in_tx(
    conn: &mut PgConnection,
    urls: &[String],
) -> Result<(), AppError> {
    sqlx::query("UPDATE images SET status = 'active' WHERE url = ANY($1)")
        .bind(urls)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn mark_images_unused_by_ids_in_tx(
    conn: &mut PgConnection,
    ids: &[i32],
) -> Result<(), AppError> {
    sqlx::query("UPDATE images SET status = 'unused' WHERE id = ANY($1)")
        .bind(ids)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

pub async fn delete_image_by_id(pool: &Pool<Postgres>, id: i32) -> Result<String, AppError> {
    let row = sqlx::query("DELETE FROM images WHERE id = $1 RETURNING storage_key")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(RequestError::NotFound)?;
    Ok(row.get("storage_key"))
}

pub async fn take_old_unused_images(pool: &Pool<Postgres>) -> Result<Vec<ImageRecord>, AppError> {
    let rows = sqlx::query(
        "DELETE FROM images WHERE status = 'unused' AND created_at < NOW() - INTERVAL '1 hour' RETURNING id, storage_key, url, status",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| ImageRecord {
            id: row.get("id"),
            storage_key: row.get("storage_key"),
            url: row.get("url"),
            status: row.get("status"),
        })
        .collect())
}
