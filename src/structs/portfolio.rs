use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PortfolioSummaryEntry {
    pub id: Uuid,
    pub member_id: i64,
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub buy_date: NaiveDate,
    pub cost_per_share: f64,
    pub shares: i64,
    pub current_price: Option<f64>,
    pub current_value: Option<f64>,
    pub pnl: Option<f64>,
    pub pnl_pct: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, FromRow)]
pub struct PortfolioEntry {
    pub id: Uuid,
    pub member_id: i64,
    pub stock_code: String,
    pub buy_date: NaiveDate,
    pub cost_per_share: f64,
    pub shares: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PortfolioRequest {
    pub stock_code: String,
    pub buy_date: NaiveDate,
    pub cost_per_share: f64,
    pub shares: i64,
}

#[derive(Serialize)]
pub struct HistoryRecord {
    pub date: NaiveDate,
    pub close: f64,
    pub adjusted_cost: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}
