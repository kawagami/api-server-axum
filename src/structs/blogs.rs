use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct PutBlog {
    pub markdown: String,
    pub html: String,
    pub tocs: Vec<Toc>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: usize, // 第幾頁
    #[serde(default = "default_per_page")]
    pub per_page: usize, // 每頁的數量
}

// 預設值函式
fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    10
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

#[derive(Serialize, Deserialize, FromRow)]
pub struct DbBlog {
    pub id: Uuid,
    pub markdown: String,
    pub html: String,
    pub tocs: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
