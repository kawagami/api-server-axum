use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 支援的彩種
pub const GAMES: &[&str] = &["lotto649", "super_lotto638"];
/// 登錄來源
pub const SOURCES: &[&str] = &["qr", "manual"];

/// 一注登錄（DB 對應）
#[derive(Clone, Serialize, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub member_id: i64,
    pub game: String,
    pub draw_date: NaiveDate,
    pub picks: Vec<i16>,
    pub second: Option<i16>,
    pub source: String, // 'qr' | 'manual'
    pub checked: bool,
    pub prize_tier: Option<String>,
    pub notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 批次登錄請求：整批共用 game / draw_date / source，notes 帶多注
#[derive(Deserialize)]
pub struct TicketBatchRequest {
    pub game: String,
    pub draw_date: NaiveDate,
    pub source: String,
    pub notes: Vec<NoteInput>,
}

/// 一注的號碼
#[derive(Deserialize)]
pub struct NoteInput {
    pub picks: Vec<i16>,
    pub second: Option<i16>,
}

/// 列表查詢
#[derive(Deserialize)]
pub struct TicketListQuery {
    pub game: Option<String>,
    pub status: Option<String>, // 'pending' | 'won' | 'lost'
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 通知偏好切換
#[derive(Deserialize)]
pub struct NotifyPrefRequest {
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct NotifyPrefResponse {
    pub enabled: bool,
}

/// 開獎結果（DB 對應，亦作 API 回傳）
#[derive(Clone, Serialize, FromRow)]
pub struct Draw {
    pub game: String,
    pub period: String,
    pub draw_date: NaiveDate,
    pub main_nums: Vec<i16>,
    pub special: i16,
}

/// 開獎查詢
#[derive(Deserialize)]
pub struct DrawListQuery {
    pub game: Option<String>,
    pub limit: Option<i64>,
}

/// 對獎 job 寄信用：一注中獎 + 收件 member email
#[derive(FromRow)]
pub struct WinnerRow {
    pub id: Uuid,
    pub member_id: i64,
    pub game: String,
    pub draw_date: NaiveDate,
    pub prize_tier: String,
    pub email: String,
}
