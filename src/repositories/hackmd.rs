use crate::{state::AppStateV2, structs::hackmd::Post};
use sqlx::QueryBuilder;

pub async fn delete_posts(state: &AppStateV2) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM hackmd_posts;")
        .execute(&state.get_pool())
        .await?;

    Ok(())
}

// bulk insert
pub async fn insert_posts_handler(state: &AppStateV2, posts: Vec<Post>) -> Result<(), sqlx::Error> {
    // 清除舊資料
    let _ = delete_posts(state).await;

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

    let query = query_builder.build();

    query.execute(&state.get_pool()).await?;

    Ok(())
}
