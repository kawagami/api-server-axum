use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 一則訪客留言(DB 列)
#[derive(Serialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// 公開端提交的留言;name / email 選填,content 必填。實際驗證/正規化在 service 層。
#[derive(Deserialize)]
pub struct NewMessage {
    pub name: Option<String>,
    pub email: Option<String>,
    pub content: String,
}

/// 後台留言分頁列表回應(同 gov_tenders 形狀)
#[derive(Serialize)]
pub struct MessagePaginatedResponse {
    pub data: Vec<Message>,
    pub total: i64,
}
