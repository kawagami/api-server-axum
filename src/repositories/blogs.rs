use crate::{errors::AppError, state::AppState, structs::blogs::DbBlog};
use sqlx::PgConnection;

/// 取得帶分頁的 blogs
pub async fn get_blogs_with_pagination(
    state: &AppState,
    limit: usize,
    offset: usize,
    tag: Option<&str>,
) -> Result<Vec<DbBlog>, AppError> {
    match tag {
        Some(t) => sqlx::query_as(
            r#"
                SELECT id, markdown, tocs, tags, created_at, updated_at
                FROM blogs
                WHERE $1 = ANY(tags)
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
        )
        .bind(t)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(state.get_pool())
        .await
        .map_err(AppError::from),
        None => sqlx::query_as(
            r#"
                SELECT id, markdown, tocs, tags, created_at, updated_at
                FROM blogs
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(state.get_pool())
        .await
        .map_err(AppError::from),
    }
}

/// 取得特定 blog
pub async fn get_blog_by_id(state: &AppState, id: uuid::Uuid) -> Result<DbBlog, AppError> {
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

pub async fn count_blogs(state: &AppState, tag: Option<&str>) -> Result<i64, AppError> {
    match tag {
        Some(t) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE $1 = ANY(tags)")
            .bind(t)
            .fetch_one(state.get_pool())
            .await
            .map_err(AppError::from),
        None => sqlx::query_scalar("SELECT COUNT(*) FROM blogs")
            .fetch_one(state.get_pool())
            .await
            .map_err(AppError::from),
    }
}

pub async fn get_all_tags(state: &AppState) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        r#"
            SELECT DISTINCT unnest(tags) AS tag
            FROM blogs
            ORDER BY tag
            "#,
    )
    .fetch_all(state.get_pool())
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
