use crate::{
    errors::AppError,
    structs::blogs::{DbBlog, TagCount},
};
use sqlx::{PgConnection, Pool, Postgres};

pub async fn get_blogs_with_pagination(
    pool: &Pool<Postgres>,
    limit: usize,
    offset: usize,
    tag: Option<&str>,
    author: Option<&str>,
    q: Option<&str>,
    ascending: bool,
) -> Result<Vec<DbBlog>, AppError> {
    // 排序方向白名單化後直接內插（不可 bind ORDER BY）；其餘條件仍走 bind 防注入
    let order = if ascending { "ASC" } else { "DESC" };
    let sql = format!(
        r#"
            SELECT b.id, b.markdown, b.tocs, b.tags, b.created_at, b.updated_at, u.name AS author_name
            FROM blogs b
            LEFT JOIN users u ON u.id = b.author_id
            WHERE ($1::text IS NULL OR $1 = ANY(b.tags))
              AND ($2::text IS NULL OR u.name = $2)
              AND ($3::text IS NULL OR b.markdown ILIKE '%' || $3 || '%')
            ORDER BY b.created_at {order}
            LIMIT $4 OFFSET $5
            "#,
    );
    sqlx::query_as(&sql)
        .bind(tag)
        .bind(author)
        .bind(q)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
}

/// 資料隔離用：取某文章的擁有者 id。外層 None = 文章不存在（＝視為新建）；內層 = author_id。
pub async fn get_author(
    pool: &Pool<Postgres>,
    id: uuid::Uuid,
) -> Result<Option<Option<i64>>, AppError> {
    let row: Option<(Option<i64>,)> = sqlx::query_as("SELECT author_id FROM blogs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(author,)| author))
}

/// 後台管理列表（依擁有者過濾）。`owner_id = None` → super_admin 看全部。公開列表不走這支。
pub async fn list_for_owner(
    pool: &Pool<Postgres>,
    owner_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<DbBlog>, AppError> {
    sqlx::query_as(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            WHERE ($1::bigint IS NULL OR author_id = $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
    )
    .bind(owner_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn count_for_owner(pool: &Pool<Postgres>, owner_id: Option<i64>) -> Result<i64, AppError> {
    sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE ($1::bigint IS NULL OR author_id = $1)")
        .bind(owner_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
}

pub async fn get_blog_by_id(pool: &Pool<Postgres>, id: uuid::Uuid) -> Result<DbBlog, AppError> {
    sqlx::query_as(
        r#"
            SELECT b.id, b.markdown, b.tocs, b.tags, b.created_at, b.updated_at, u.name AS author_name
            FROM blogs b
            LEFT JOIN users u ON u.id = b.author_id
            WHERE b.id = $1
            "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn count_blogs(
    pool: &Pool<Postgres>,
    tag: Option<&str>,
    author: Option<&str>,
    q: Option<&str>,
) -> Result<i64, AppError> {
    sqlx::query_scalar(
        r#"
            SELECT COUNT(*)
            FROM blogs b
            LEFT JOIN users u ON u.id = b.author_id
            WHERE ($1::text IS NULL OR $1 = ANY(b.tags))
              AND ($2::text IS NULL OR u.name = $2)
              AND ($3::text IS NULL OR b.markdown ILIKE '%' || $3 || '%')
            "#,
    )
    .bind(tag)
    .bind(author)
    .bind(q)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_all_tags(pool: &Pool<Postgres>) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        r#"
            SELECT DISTINCT unnest(tags) AS tag
            FROM blogs
            ORDER BY tag
            "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

/// 每個 tag 的文章數（公開列表側欄用）；依 tag 字母排序
pub async fn get_tag_counts(pool: &Pool<Postgres>) -> Result<Vec<TagCount>, AppError> {
    sqlx::query_as(
        r#"
            SELECT tag, COUNT(*) AS count
            FROM blogs, unnest(tags) AS tag
            GROUP BY tag
            ORDER BY tag
            "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn delete_blog_in_tx(conn: &mut PgConnection, id: uuid::Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM blogs WHERE id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

pub async fn upsert_blog_in_tx(
    conn: &mut PgConnection,
    id: uuid::Uuid,
    markdown: String,
    tocs: Vec<String>,
    tags: Vec<String>,
    author_id: i64,
) -> Result<(), AppError> {
    // author_id 只在 INSERT 生效；ON CONFLICT 的 UPDATE 不動它 → 保留原始擁有者
    let query = r#"
            INSERT INTO blogs (id, markdown, tocs, tags, author_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE SET
                markdown = EXCLUDED.markdown,
                tocs = EXCLUDED.tocs,
                tags = EXCLUDED.tags,
                updated_at = NOW();
        "#;

    sqlx::query(query)
        .bind(id)
        .bind(markdown)
        .bind(tocs)
        .bind(tags)
        .bind(author_id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
