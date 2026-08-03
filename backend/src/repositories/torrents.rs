use crate::{
    errors::{AppError, RequestError},
    structs::pagination::Paginated,
    structs::torrents::{Torrent, STATUS_DOWNLOADING, STATUS_PENDING}
};
use sqlx::{Pool, Postgres};

const COLUMNS: &str = "id, info_hash, magnet_uri, name, status, total_size, files, error, created_by, created_at, completed_at";

/// `list` 內 data 與 total 兩個查詢共用的篩選條件（$1..$2）。**兩邊 bind 順序必須一致**。
const LIST_FILTER: &str = "($1::text IS NULL OR status = $1)
           AND ($2::bigint IS NULL OR owner_id = $2)";

pub async fn insert(
    pool: &Pool<Postgres>,
    info_hash: &str,
    magnet_uri: &str,
    created_by: &str,
    owner_id: Option<i64>,
) -> Result<Torrent, AppError> {
    sqlx::query_as::<_, Torrent>(&format!(
        "INSERT INTO torrents (info_hash, magnet_uri, created_by, owner_id) VALUES ($1, $2, $3, $4) RETURNING {COLUMNS}"
    ))
    .bind(info_hash)
    .bind(magnet_uri)
    .bind(created_by)
    .bind(owner_id)
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

/// 資料隔離用：取某任務的擁有者 id；任務不存在回 NotFound。
pub async fn get_owner(pool: &Pool<Postgres>, id: i32) -> Result<Option<i64>, AppError> {
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT owner_id FROM torrents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    row.map(|(owner,)| owner).ok_or_else(|| RequestError::NotFound.into())
}

/// `owner_id = None` → 不過濾（super_admin 看全部）；`Some(id)` → 只列該擁有者。
pub async fn list(
    pool: &Pool<Postgres>,
    status: Option<String>,
    owner_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Torrent>, AppError> {
    let data = sqlx::query_as::<_, Torrent>(&format!(
        "SELECT {COLUMNS} FROM torrents
         WHERE {LIST_FILTER}
         ORDER BY id DESC LIMIT $3 OFFSET $4"
    ))
    .bind(&status)
    .bind(owner_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM torrents WHERE {LIST_FILTER}"
    ))
    .bind(&status)
    .bind(owner_id)
    .fetch_one(pool)
    .await?;

    Ok(Paginated::new(data, total))
}

/// 取可啟動的任務：pending（排隊中）與 downloading（重啟後待 resume）。
/// 排序：續傳的 downloading 最優先 → 沒試過的 → 最久沒試的（冷門種子逾時後會沉到隊尾，
/// 不會每輪都搶在新任務前面）。
pub async fn list_resumable(pool: &Pool<Postgres>, limit: i64) -> Result<Vec<Torrent>, AppError> {
    Ok(sqlx::query_as::<_, Torrent>(&format!(
        "SELECT {COLUMNS} FROM torrents WHERE status IN ($1, $2)
         ORDER BY (status = 'downloading') DESC, last_attempt_at ASC NULLS FIRST, id LIMIT $3"
    ))
    .bind(STATUS_PENDING)
    .bind(STATUS_DOWNLOADING)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// 待啟動任務總數（pending + downloading）— 與併發上限比較就知道有沒有任務排不進名額
pub async fn count_resumable(pool: &Pool<Postgres>) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT count(*) FROM torrents WHERE status IN ($1, $2)")
            .bind(STATUS_PENDING)
            .bind(STATUS_DOWNLOADING)
            .fetch_one(pool)
            .await?,
    )
}

/// 記一次啟動嘗試，回傳這是第幾次（排序用 + 判斷還有沒有重試額度）
pub async fn mark_attempt(pool: &Pool<Postgres>, id: i32) -> Result<i32, AppError> {
    Ok(sqlx::query_scalar(
        "UPDATE torrents SET last_attempt_at = now(), attempt_count = attempt_count + 1
         WHERE id = $1 RETURNING attempt_count",
    )
    .bind(id)
    .fetch_one(pool)
    .await?)
}

/// metadata 逾時但還有重試額度：留在 pending（已被 mark_attempt 排到隊尾），只記下原因
pub async fn set_retry_pending(pool: &Pool<Postgres>, id: i32, error: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE torrents SET status = 'pending', error = $2 WHERE id = $1")
        .bind(id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_downloading_metadata(
    pool: &Pool<Postgres>,
    id: i32,
    name: &str,
    total_size: i64,
    files: &serde_json::Value,
) -> Result<(), AppError> {
    // attempt_count 歸零：metadata 已到手，之後若失敗重跑要重新給滿重試額度
    sqlx::query(
        "UPDATE torrents SET status = 'downloading', name = $2, total_size = $3, files = $4,
                error = NULL, attempt_count = 0 WHERE id = $1",
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

/// 重設為 pending（重試）。回傳是否有更新到（id 不存在或仍在下載中 → false）。
/// 清掉嘗試紀錄 —— 手動重設是明確要求重跑，排到候選最前面。
pub async fn reset_pending(pool: &Pool<Postgres>, id: i32) -> Result<bool, AppError> {
    let result = sqlx::query(
        "UPDATE torrents SET status = 'pending', error = NULL, completed_at = NULL,
                attempt_count = 0, last_attempt_at = NULL
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
