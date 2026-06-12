use crate::{
    errors::{AppError, RequestError},
    structs::torrents::{Torrent, TorrentPaginatedResponse, STATUS_DOWNLOADING, STATUS_PENDING},
};
use sqlx::{Pool, Postgres};

const COLUMNS: &str = "id, info_hash, magnet_uri, name, status, total_size, files, error, created_by, created_at, completed_at";

pub async fn insert(
    pool: &Pool<Postgres>,
    info_hash: &str,
    magnet_uri: &str,
    created_by: &str,
) -> Result<Torrent, AppError> {
    sqlx::query_as::<_, Torrent>(&format!(
        "INSERT INTO torrents (info_hash, magnet_uri, created_by) VALUES ($1, $2, $3) RETURNING {COLUMNS}"
    ))
    .bind(info_hash)
    .bind(magnet_uri)
    .bind(created_by)
    .fetch_one(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            RequestError::Conflict("相同 info_hash 的任務已存在".to_string()).into()
        }
        _ => e.into(),
    })
}

pub async fn get_by_id(pool: &Pool<Postgres>, id: i32) -> Result<Torrent, AppError> {
    Ok(
        sqlx::query_as::<_, Torrent>(&format!("SELECT {COLUMNS} FROM torrents WHERE id = $1"))
            .bind(id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn list(
    pool: &Pool<Postgres>,
    status: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<TorrentPaginatedResponse, AppError> {
    let data = sqlx::query_as::<_, Torrent>(&format!(
        "SELECT {COLUMNS} FROM torrents
         WHERE ($1::text IS NULL OR status = $1)
         ORDER BY id DESC LIMIT $2 OFFSET $3"
    ))
    .bind(&status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM torrents WHERE ($1::text IS NULL OR status = $1)")
            .bind(&status)
            .fetch_one(pool)
            .await?;

    Ok(TorrentPaginatedResponse { data, total })
}

/// 取可啟動的任務：pending（排隊中）與 downloading（重啟後待 resume），舊的優先
pub async fn list_resumable(pool: &Pool<Postgres>, limit: i64) -> Result<Vec<Torrent>, AppError> {
    Ok(sqlx::query_as::<_, Torrent>(&format!(
        "SELECT {COLUMNS} FROM torrents WHERE status IN ($1, $2) ORDER BY id LIMIT $3"
    ))
    .bind(STATUS_PENDING)
    .bind(STATUS_DOWNLOADING)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn set_downloading_metadata(
    pool: &Pool<Postgres>,
    id: i32,
    name: &str,
    total_size: i64,
    files: &serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE torrents SET status = 'downloading', name = $2, total_size = $3, files = $4, error = NULL WHERE id = $1",
    )
    .bind(id)
    .bind(name)
    .bind(total_size)
    .bind(files)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_completed(pool: &Pool<Postgres>, id: i32) -> Result<(), AppError> {
    sqlx::query("UPDATE torrents SET status = 'completed', completed_at = now(), error = NULL WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_failed(pool: &Pool<Postgres>, id: i32, error: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE torrents SET status = 'failed', error = $2 WHERE id = $1")
        .bind(id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}

/// 重設為 pending（重試）。回傳是否有更新到（id 不存在或仍在下載中 → false）
pub async fn reset_pending(pool: &Pool<Postgres>, id: i32) -> Result<bool, AppError> {
    let result = sqlx::query(
        "UPDATE torrents SET status = 'pending', error = NULL, completed_at = NULL
         WHERE id = $1 AND status IN ('failed', 'completed')",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// 刪除並回傳 info_hash 供磁碟清理；不存在 → NotFound
pub async fn delete(pool: &Pool<Postgres>, id: i32) -> Result<String, AppError> {
    let info_hash: Option<String> =
        sqlx::query_scalar("DELETE FROM torrents WHERE id = $1 RETURNING info_hash")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    info_hash.ok_or_else(|| RequestError::NotFound.into())
}

/// 已知大小總和（bytes）— 收新任務前的容量檢查
pub async fn total_size_sum(pool: &Pool<Postgres>) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COALESCE(SUM(total_size), 0)::bigint FROM torrents")
            .fetch_one(pool)
            .await?,
    )
}

/// 逾期任務：completed 超過保留天數，或 failed 超過保留天數（以 created_at 計）
pub async fn list_expired(pool: &Pool<Postgres>, retention_days: i64) -> Result<Vec<Torrent>, AppError> {
    Ok(sqlx::query_as::<_, Torrent>(&format!(
        "SELECT {COLUMNS} FROM torrents
         WHERE (status = 'completed' AND completed_at < now() - ($1 || ' days')::interval)
            OR (status = 'failed' AND created_at < now() - ($1 || ' days')::interval)"
    ))
    .bind(retention_days.to_string())
    .fetch_all(pool)
    .await?)
}
