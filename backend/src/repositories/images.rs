use crate::errors::{AppError, RequestError};
use crate::structs::images::ImageRecord;
use sqlx::{PgConnection, Pool, Postgres};

const RECORD_COLUMNS: &str = "id, storage_key, url, status, width, height, blur_data_url";

/// `owner_id = None` → 不過濾（super_admin 看全部）；`Some(id)` → 只列該擁有者。
pub async fn get_all_images(
    pool: &Pool<Postgres>,
    owner_id: Option<i64>,
) -> Result<Vec<ImageRecord>, AppError> {
    Ok(sqlx::query_as(&format!(
        "SELECT {RECORD_COLUMNS} FROM images
         WHERE ($1::bigint IS NULL OR owner_id = $1)
         ORDER BY id DESC"
    ))
    .bind(owner_id)
    .fetch_all(pool)
    .await?)
}

/// 資料隔離用：取某圖片的擁有者 id；不存在回 NotFound。
pub async fn get_owner(pool: &Pool<Postgres>, id: i32) -> Result<Option<i64>, AppError> {
    let row: Option<(Option<i64>,)> = sqlx::query_as("SELECT owner_id FROM images WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    row.map(|(owner,)| owner).ok_or_else(|| RequestError::NotFound.into())
}

pub struct NewImage<'a> {
    pub storage_key: &'a str,
    pub url: &'a str,
    pub owner_id: Option<i64>,
    pub width: i32,
    pub height: i32,
    pub blur_data_url: Option<&'a str>,
}

pub async fn insert_image(pool: &Pool<Postgres>, image: NewImage<'_>) -> Result<ImageRecord, AppError> {
    Ok(sqlx::query_as(&format!(
        "INSERT INTO images (storage_key, url, owner_id, width, height, blur_data_url)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING {RECORD_COLUMNS}"
    ))
    .bind(image.storage_key)
    .bind(image.url)
    .bind(image.owner_id)
    .bind(image.width)
    .bind(image.height)
    .bind(image.blur_data_url)
    .fetch_one(pool)
    .await?)
}

pub async fn get_images_by_urls(
    pool: &Pool<Postgres>,
    urls: &[String],
) -> Result<Vec<ImageRecord>, AppError> {
    Ok(sqlx::query_as(&format!("SELECT {RECORD_COLUMNS} FROM images WHERE url = ANY($1)"))
        .bind(urls)
        .fetch_all(pool)
        .await?)
}

/// 已有版位資訊的圖片（回填前 / decode 失敗的列不含），供 blog 內文的模糊預覽。
pub async fn get_placeholders_by_urls(
    pool: &Pool<Postgres>,
    urls: &[String],
) -> Result<Vec<(String, i32, i32, Option<String>)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT url, width, height, blur_data_url FROM images
         WHERE url = ANY($1) AND width IS NOT NULL AND height IS NOT NULL",
    )
    .bind(urls)
    .fetch_all(pool)
    .await?)
}

/// 回填用：取 `id > after_id` 且尚無寬高的圖片 (id, storage_key)，依 id 遞增。
pub async fn list_missing_placeholders(
    pool: &Pool<Postgres>,
    after_id: i32,
    limit: i64,
) -> Result<Vec<(i32, String)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, storage_key FROM images
         WHERE width IS NULL AND id > $1
         ORDER BY id LIMIT $2",
    )
    .bind(after_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn set_placeholder(
    pool: &Pool<Postgres>,
    id: i32,
    width: i32,
    height: i32,
    blur_data_url: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE images SET width = $2, height = $3, blur_data_url = $4 WHERE id = $1")
        .bind(id)
        .bind(width)
        .bind(height)
        .bind(blur_data_url)
        .execute(pool)
        .await?;
    Ok(())
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
    let (storage_key,): (String,) = sqlx::query_as("DELETE FROM images WHERE id = $1 RETURNING storage_key")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(RequestError::NotFound)?;
    Ok(storage_key)
}

pub async fn take_old_unused_images(pool: &Pool<Postgres>) -> Result<Vec<ImageRecord>, AppError> {
    Ok(sqlx::query_as(&format!(
        "DELETE FROM images WHERE status = 'unused' AND created_at < NOW() - INTERVAL '1 hour'
         RETURNING {RECORD_COLUMNS}"
    ))
    .fetch_all(pool)
    .await?)
}
