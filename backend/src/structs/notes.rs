use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct LastChangeUser {
    pub biography: Option<String>,
    pub name: String,
    pub photo: String,
    #[serde(rename = "userPath")]
    pub user_path: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Post {
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    pub id: String,
    #[serde(rename = "lastChangeUser")]
    pub last_change_user: LastChangeUser,
    #[serde(rename = "lastChangedAt")]
    pub last_changed_at: i64,
    pub permalink: Option<String>,
    #[serde(rename = "publishLink")]
    pub publish_link: Option<String>,
    #[serde(rename = "publishType")]
    pub publish_type: String,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<i64>,
    #[serde(rename = "readPermission")]
    pub read_permission: String,
    #[serde(rename = "shortId")]
    pub short_id: String,
    pub tags: Vec<String>,
    #[serde(rename = "tagsUpdatedAt")]
    pub tags_updated_at: Option<i64>,
    #[serde(rename = "teamPath")]
    pub team_path: Option<String>,
    pub title: String,
    #[serde(rename = "titleUpdatedAt")]
    pub title_updated_at: i64,
    #[serde(rename = "userPath")]
    pub user_path: String,
    #[serde(rename = "writePermission")]
    pub write_permission: String,
}

#[derive(Serialize, FromRow)]
pub struct HackmdNoteListAndTag {
    pub id: String,
    pub title: String,
    pub publish_link: String,
    pub last_changed_at: i64,
    pub read_permission: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
