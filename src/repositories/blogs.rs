use crate::{errors::AppError, state::AppStateV2, structs::blogs::DbBlog};
use sqlx::PgConnection;

/// 取得帶分頁的 blogs
pub async fn get_blogs_with_pagination(
    state: &AppStateV2,
    limit: usize,
    offset: usize,
) -> Result<Vec<DbBlog>, AppError> {
    sqlx::query_as(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
    )
    .bind(limit as i64) // 將限制數量綁定到查詢
    .bind(offset as i64) // 將偏移量綁定到查詢
    .fetch_all(state.get_pool())
    .await
    .map_err(AppError::from)
}

/// 取得特定 blog
pub async fn get_blog_by_id(state: &AppStateV2, id: uuid::Uuid) -> Result<DbBlog, AppError> {
    sqlx::query_as(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            WHERE id = $1
            "#,
    )
    .bind(id)
    .fetch_one(state.get_pool())
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
) -> Result<(), AppError> {
    let query = r#"
            INSERT INTO blogs (id, markdown, tocs, tags, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
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
        .execute(&mut *conn)
        .await?;

    Ok(())
}
