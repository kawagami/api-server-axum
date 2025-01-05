use crate::{state::AppStateV2, structs::blogs::DbBlog};

/// 取得帶分頁的 blogs
pub async fn get_blogs_with_pagination(
    state: &AppStateV2,
    limit: usize,
    offset: usize,
) -> Result<Vec<DbBlog>, sqlx::Error> {
    let blogs = sqlx::query_as::<_, DbBlog>(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
    )
    .bind(limit as i64) // 將限制數量綁定到查詢
    .bind(offset as i64) // 將偏移量綁定到查詢
    .fetch_all(&state.get_pool())
    .await?;

    Ok(blogs)
}

/// 取得特定 blog
pub async fn get_blog_by_id(state: &AppStateV2, id: uuid::Uuid) -> Result<DbBlog, sqlx::Error> {
    let blog = sqlx::query_as::<_, DbBlog>(
        r#"
            SELECT id, markdown, tocs, tags, created_at, updated_at
            FROM blogs
            WHERE id = $1
            "#,
    )
    .bind(id)
    .fetch_one(&state.get_pool())
    .await?;

    Ok(blog)
}

/// 刪除特定 blog
pub async fn delete_blog(state: &AppStateV2, id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
            DELETE FROM blogs
            WHERE id = $1
            "#,
    )
    .bind(id)
    .execute(&state.get_pool())
    .await?;

    Ok(())
}

/// insert or update blog
pub async fn upsert_blog(
    state: &AppStateV2,
    id: uuid::Uuid,
    markdown: String,
    tocs: Vec<String>,
    tags: Vec<String>,
) -> Result<(), sqlx::Error> {
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
        .bind(id) // $1
        .bind(markdown) // $2
        .bind(tocs) // $3
        .bind(tags) // $4
        .execute(&state.get_pool())
        .await?;

    Ok(())
}
