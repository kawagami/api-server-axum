use crate::{
    errors::AppError,
    structs::notes::{HackmdNoteListAndTag, Post, Tag},
};
use sqlx::{Pool, Postgres, QueryBuilder};

pub async fn delete_posts(pool: &Pool<Postgres>) -> Result<(), AppError> {
    sqlx::query("DELETE FROM hackmd_posts;")
        .execute(pool)
        .await
        .map_err(AppError::from)?;

    Ok(())
}

pub async fn insert_posts_handler(pool: &Pool<Postgres>, posts: Vec<Post>) -> Result<(), AppError> {
    let _ = delete_posts(pool).await;

    let mut query_builder = QueryBuilder::new(
        r#"
        INSERT INTO hackmd_posts (
            id, content, created_at, last_changed_at, user_path,
            permalink, publish_link, publish_type, published_at,
            read_permission, short_id, tags, tags_updated_at,
            team_path, title, title_updated_at, write_permission
        )
        "#,
    );

    query_builder.push_values(posts, |mut b, post| {
        b.push_bind(post.id)
            .push_bind(post.content)
            .push_bind(post.created_at)
            .push_bind(post.last_changed_at)
            .push_bind(post.user_path)
            .push_bind(post.permalink)
            .push_bind(post.publish_link)
            .push_bind(post.publish_type)
            .push_bind(post.published_at)
            .push_bind(post.read_permission)
            .push_bind(post.short_id)
            .push_bind(post.tags)
            .push_bind(post.tags_updated_at)
            .push_bind(post.team_path)
            .push_bind(post.title)
            .push_bind(post.title_updated_at)
            .push_bind(post.write_permission);
    });

    query_builder.build().execute(pool).await.map_err(AppError::from)?;

    Ok(())
}

pub async fn get_tags(pool: &Pool<Postgres>) -> Result<Vec<Tag>, AppError> {
    sqlx::query_as(
        r#"
            SELECT
                ROW_NUMBER() OVER (ORDER BY MAX(last_changed_at) DESC) AS id,
                name
            FROM (
                SELECT
                    unnest(tags) AS name,
                    last_changed_at
                FROM hackmd_posts
            ) subquery
            GROUP BY name
            ORDER BY MAX(last_changed_at) DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

pub async fn get_lists(pool: &Pool<Postgres>) -> Result<Vec<HackmdNoteListAndTag>, AppError> {
    sqlx::query_as(
        r#"
            SELECT
                id,
                title,
                publish_link,
                last_changed_at,
                read_permission,
                tags
            FROM
                hackmd_posts
         	WHERE NOT (tags @> ARRAY['工作']) AND read_permission='guest'
            ORDER BY
                last_changed_at DESC;
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}
