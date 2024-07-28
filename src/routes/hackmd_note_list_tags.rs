use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Serialize;

use crate::state::SharedState;

#[derive(Serialize, sqlx::FromRow)]
pub struct Tag {
    id: i64,
    name: String,
}

pub async fn get_all_note_list_tags(
    State(state): State<SharedState>,
) -> Result<Json<Vec<Tag>>, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = r#"
            SELECT id, name FROM hackmd_tags
        "#;
    let records = sqlx::query_as::<_, Tag>(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(records))
}