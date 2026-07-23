use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 一則 blog 留言(讀取視圖;member_id 有值時 author_name/avatar_url 取自 members 表)。
/// `is_member` 讓前端可標示會員身分;`author_name` 為顯示名(訪客可能為 None,前端 fallback「訪客」)。
#[derive(Serialize, FromRow)]
pub struct BlogComment {
    pub id: i64,
    pub blog_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub is_member: bool,
    pub author_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// 公開端提交的留言;content 必填,name 為訪客自填顯示名(選填,會員留言忽略此欄)。
/// 實際驗證/正規化在 service 層。
#[derive(Deserialize)]
pub struct NewComment {
    pub content: String,
    pub name: Option<String>,
}

/// 留言分頁列表回應(公開列表與後台列表共用,同 messages 形狀)
#[derive(Serialize)]
pub struct BlogCommentPaginatedResponse {
    pub data: Vec<BlogComment>,
    pub total: i64,
}
