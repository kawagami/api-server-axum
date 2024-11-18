use crate::state::AppStateV2;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Serialize;

#[derive(Serialize, sqlx::FromRow)]
pub struct Tag {
    id: i64,
    name: String,
}

pub async fn get_all_note_list_tags(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<Tag>>, (StatusCode, String)> {
    let pool = &state.get_pool().await;
    let query = r#"
            SELECT id, name FROM hackmd_tags
        "#;
    let records: Vec<Tag> = sqlx::query_as(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(records))
}
