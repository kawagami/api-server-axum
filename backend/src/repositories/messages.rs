use crate::{errors::AppError, structs::messages::Message};
use sqlx::{Pool, Postgres};

const COLS: &str = "id, name, email, content, created_at";

/// 寫入一則留言,回傳完整列
pub async fn insert(
    pool: &Pool<Postgres>,
    name: Option<&str>,
    email: Option<&str>,
    content: &str,
) -> Result<Message, AppError> {
    let row = sqlx::query_as(&format!(
        "INSERT INTO messages (name, email, content)
         VALUES ($1, $2, $3)
         RETURNING {COLS}"
    ))
    .bind(name)
    .bind(email)
    .bind(content)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// 留言分頁列表(新到舊)
pub async fn list(pool: &Pool<Postgres>, limit: i64, offset: i64) -> Result<Vec<Message>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM messages ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2"
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count(pool: &Pool<Postgres>) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages")
        .fetch_one(pool)
        .await?;
    Ok(total)
}

/// 批次刪除，回傳實際刪掉的 id（不存在的略過，不視為錯誤）
pub async fn delete_many(pool: &Pool<Postgres>, ids: &[i64]) -> Result<Vec<i64>, AppError> {
    Ok(sqlx::query_scalar("DELETE FROM messages WHERE id = ANY($1) RETURNING id")
        .bind(ids)
        .fetch_all(pool)
        .await?)
}
