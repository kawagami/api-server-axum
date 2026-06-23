use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Member {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MemberDetail {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub providers: Vec<String>,
    pub lottery_notify_enabled: bool, // 統一發票中獎 email 通知開關
}

#[derive(Clone, Debug)]
pub struct AuthenticatedMember {
    pub member_id: i64,
}

#[derive(Deserialize)]
pub struct ExchangeCodeRequest {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}
