use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct PutBlog {
    pub markdown: String,
    pub tocs: Vec<Toc>,
    pub tags: Vec<String>,
}

/// blogs 列表的標籤過濾（分頁走共用 `PageQuery`）。
#[derive(Deserialize)]
pub struct TagFilter {
    pub tag: Option<String>,
}

impl PutBlog {
    /// 提取 tocs 中的 text 字段，返回 Vec<String>
    pub fn extract_toc_texts(&self) -> Vec<String> {
        self.tocs.iter().map(|toc| toc.text.clone()).collect()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Toc {
    id: String,
    level: u32,
    text: String,
}

#[derive(Serialize)]
pub struct BlogsResponse {
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub data: Vec<DbBlog>,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct DbBlog {
    pub id: Uuid,
    pub markdown: String,
    pub tocs: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
