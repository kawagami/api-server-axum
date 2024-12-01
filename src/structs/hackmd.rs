use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;

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

#[derive(Serialize, sqlx::FromRow)]
pub struct HackmdNoteListAndTag {
    pub id: String,
    pub title: String,
    pub publish_link: String,
    pub last_changed_at: i64,
    pub read_permission: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
