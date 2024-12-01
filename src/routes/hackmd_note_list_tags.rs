use crate::{state::AppStateV2, structs::hackmd::Tag};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};

pub async fn get_all_note_list_tags(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<Tag>>, (StatusCode, String)> {
    let query = r#"
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
        "#;
    let records: Vec<Tag> = sqlx::query_as(query)
        .fetch_all(&state.get_pool())
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(records))
}
