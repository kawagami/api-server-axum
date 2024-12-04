use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct FirebaseImage {
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbFirebaseImage {
    pub id: i32,
    pub image_url: String,
    pub created_at: DateTime<Utc>, // 支援 TIMESTAMPTZ 型別
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub files: Vec<Image>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteImageRequest {
    pub file_name: String,
}
