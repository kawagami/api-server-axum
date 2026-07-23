use crate::{errors::AppError, structs::blog_comments::BlogComment};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

// 讀取視圖共用欄位:LEFT JOIN members 取會員顯示名/頭像,訪客則落回 author_name
const SELECT_COLS: &str = "c.id, c.blog_id, c.content, c.created_at, \
     (c.member_id IS NOT NULL) AS is_member, \
     COALESCE(m.name, c.author_name) AS author_name, \
     m.avatar_url AS avatar_url";

/// blog 是否存在(POST 前檢查,不存在回 404 而非讓 FK 噴 500)
pub async fn blog_exists(pool: &Pool<Postgres>, blog_id: Uuid) -> Result<bool, AppError> {
    let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM blogs WHERE id = $1")
        .bind(blog_id)
        .fetch_optional(pool)
        .await?;
    Ok(exists.is_some())
}

/// 寫入一則留言,回傳含 join 顯示名/頭像的完整視圖
pub async fn insert(
    pool: &Pool<Postgres>,
    blog_id: Uuid,
    member_id: Option<i64>,
    author_name: Option<&str>,
    content: &str,
) -> Result<BlogComment, AppError> {
    let row = sqlx::query_as(&format!(
        "WITH ins AS (
             INSERT INTO blog_comments (blog_id, member_id, author_name, content)
             VALUES ($1, $2, $3, $4)
             RETURNING id, blog_id, member_id, author_name, content, created_at
         )
         SELECT {SELECT_COLS}
         FROM ins c LEFT JOIN members m ON m.id = c.member_id"
    ))
    .bind(blog_id)
    .bind(member_id)
    .bind(author_name)
    .bind(content)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// 單篇 blog 的留言分頁(舊到新:留言區順讀)
pub async fn list_by_blog(
    pool: &Pool<Postgres>,
    blog_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<BlogComment>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS}
         FROM blog_comments c LEFT JOIN members m ON m.id = c.member_id
         WHERE c.blog_id = $1
         ORDER BY c.created_at ASC, c.id ASC
         LIMIT $2 OFFSET $3"
    ))
    .bind(blog_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count_by_blog(pool: &Pool<Postgres>, blog_id: Uuid) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blog_comments WHERE blog_id = $1")
        .bind(blog_id)
        .fetch_one(pool)
        .await?;
    Ok(total)
}

/// 後台:全站留言分頁(新到舊,便於巡檢最新留言)
pub async fn list_all(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BlogComment>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {SELECT_COLS}
         FROM blog_comments c LEFT JOIN members m ON m.id = c.member_id
         ORDER BY c.created_at DESC, c.id DESC
         LIMIT $1 OFFSET $2"
    ))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn count_all(pool: &Pool<Postgres>) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM blog_comments")
        .fetch_one(pool)
        .await?;
    Ok(total)
}

/// 刪除一則留言;不存在回 RowNotFound(→ 404)
pub async fn delete(pool: &Pool<Postgres>, id: i64) -> Result<(), AppError> {
    let res = sqlx::query("DELETE FROM blog_comments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(crate::errors::RequestError::NotFound.into());
    }
    Ok(())
}
