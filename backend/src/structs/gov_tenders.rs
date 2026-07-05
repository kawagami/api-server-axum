use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct GovTender {
    pub id: i64,
    pub filename: String,
    pub date: NaiveDate,
    pub tender_type: String,
    pub title: String,
    pub category: Option<String>,
    pub unit_id: String,
    pub unit_name: String,
    pub job_number: String,
    pub companies: serde_json::Value,
    pub keyword: String,
    pub detail_url: String,
    pub notified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// 從來源 API 解析出、待寫入 DB 的一筆標案
pub struct NewGovTender {
    pub filename: String,
    pub date: NaiveDate,
    pub tender_type: String,
    pub title: String,
    pub category: Option<String>,
    pub unit_id: String,
    pub unit_name: String,
    pub job_number: String,
    pub companies: Vec<String>,
    pub keyword: String,
    pub detail_url: String,
}

#[derive(Deserialize)]
pub struct GovTenderListQuery {
    pub keyword: Option<String>,
    pub tender_type: Option<String>,
    /// 標案名稱 / 機關名稱模糊搜尋
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct GovTenderPaginatedResponse {
    pub data: Vec<GovTender>,
    pub total: i64,
}
