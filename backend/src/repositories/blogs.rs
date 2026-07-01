use crate::{errors::AppError, structs::blogs::DbBlog};
use sqlx::{PgConnection, Pool, Postgres};

pub async fn get_blogs_with_pagination(
    pool: &Pool<Postgres>,
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
        .fetch_all(pool)
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
        .fetch_all(pool)
        .await
        .map_err(AppError::from),
    }
}

pub async fn get_blog_by_id(pool: &Pool<Postgres>, id: uuid::Uuid) -> Result<DbBlog, AppError> {
    sqlx::query_as(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            WHERE id = $1
            "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

pub async fn count_blogs(pool: &Pool<Postgres>, tag: Option<&str>) -> Result<i64, AppError> {
    match tag {
        Some(t) => sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE $1 = ANY(tags)")
            .bind(t)
            .fetch_one(pool)
            .await
            .map_err(AppError::from),
        None => sqlx::query_scalar("SELECT COUNT(*) FROM blogs")
            .fetch_one(pool)
            .await
            .map_err(AppError::from),
    }
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
