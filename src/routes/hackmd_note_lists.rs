use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;

use crate::state::SharedState;

#[derive(Serialize, sqlx::FromRow)]
pub struct HackmdNoteList {
    id: i64,
    is_public: bool,
    hackmd_note_lists_id: String,
    title: String,
    #[sqlx(rename = "createdAt")]
    created_at_hackmd: i64,
    #[sqlx(rename = "publishType")]
    publish_type: String,
    #[sqlx(rename = "publishedAt")]
    published_at: Option<i64>,
    permalink: Option<String>,
    #[sqlx(rename = "publishLink")]
    publish_link: String,
    #[sqlx(rename = "shortId")]
    short_id: String,
    #[sqlx(rename = "lastChangedAt")]
    last_changed_at: i64,
    #[sqlx(rename = "lastChangeUser")]
    last_change_user: sqlx::types::Json<LastChangeUser>,
    #[sqlx(rename = "userPath")]
    user_path: String,
    #[sqlx(rename = "teamPath")]
    team_path: Option<String>,
    #[sqlx(rename = "readPermission")]
    read_permission: String,
    #[sqlx(rename = "writePermission")]
    write_permission: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
struct LastChangeUser {
    name: String,
    photo: String,
    biography: Option<String>,
    #[sqlx(rename = "userPath")]
    user_path: Option<String>,
}

pub async fn get_note_list(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Result<Json<HackmdNoteList>, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = "select * from hackmd_note_lists where id = $1";
    let result = sqlx::query_as::<_, HackmdNoteList>(query)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(result))
}

pub async fn get_all_note_lists(
    State(state): State<SharedState>,
) -> Result<Json<Vec<HackmdNoteList>>, (StatusCode, String)> {
    let pool = &state.read().unwrap().pool.clone();
    let query = "select * from hackmd_note_lists";
    let records = sqlx::query_as::<_, HackmdNoteList>(query)
        .fetch_all(pool)
        .await
        .map_err(|err| (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()))?;

    Ok(Json(records))
}
