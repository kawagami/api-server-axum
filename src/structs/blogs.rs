use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct CreateBlog {
    pub id: Uuid,
    pub markdown: String,
    pub html: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct DbBlog {
    pub id: Uuid,
    pub markdown: String,
    pub html: String,
    pub tags: Vec<Option<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
