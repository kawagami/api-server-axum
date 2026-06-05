use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// HTML 解析 / buyback API 輸入用（民國日期字串）
#[derive(Serialize, Deserialize, FromRow)]
pub struct StockRequest {
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
}

/// stock_changes 資料列的識別 key（西元 NaiveDate）
#[derive(Debug, Clone)]
pub struct StockChangeRef {
    pub stock_no: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct StockChange {
    pub id: Option<i32>,
    pub stock_no: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: Option<String>,
    pub stock_name: Option<String>,
    pub start_price: Option<f64>,
    pub end_price: Option<f64>,
    pub change: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockChangeId {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct Conditions {
    pub status: Option<String>,
    #[serde(default = "default_changes_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_changes_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct StockChangePaginatedResponse {
    pub data: Vec<StockChange>,
    pub total: i64,
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

#[derive(Serialize, Deserialize)]
pub struct StockDayAvgResponse {
    pub stat: String,
    pub title: String,
    pub fields: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub notes: Vec<String>,
}

#[derive(Serialize, Clone, FromRow, Debug)]
pub struct NewStockClosingPrice {
    pub stock_no: String,
    pub date: NaiveDate,
    pub close_price: f64,
}

#[derive(Serialize)]
pub struct StockClosingPriceResponse {
    pub prices: (NewStockClosingPrice, NewStockClosingPrice),
    pub stats: StockStats,
}

#[derive(Serialize)]
pub struct StockStats {
    pub price_diff: f64,
    pub percent_change: f64,
    pub is_increase: bool,
    pub day_span: i64,
}

#[derive(Deserialize, Debug)]
pub struct TwseApiResponse {
    pub date: String,
    pub data: Vec<Vec<String>>,
}

#[derive(Deserialize)]
pub struct GetStockDayAll {
    pub trade_date: Option<NaiveDate>,
    pub stock_code: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct StockDayAll {
    pub id: i32,
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub stock_name: String,
    pub trade_volume: Option<i64>,
    pub trade_amount: Option<i64>,
    pub open_price: Option<Decimal>,
    pub high_price: Option<Decimal>,
    pub low_price: Option<Decimal>,
    pub close_price: Option<Decimal>,
    pub price_change: Option<Decimal>,
    pub transaction_count: Option<i32>,
}

pub struct StockDayAllInsertRow {
    pub trade_date: NaiveDate,
    pub stock_code: String,
    pub stock_name: String,
    pub trade_volume: i64,
    pub trade_amount: i64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub close_price: f64,
    pub price_change: f64,
    pub transaction_count: i32,
}

#[derive(Debug, FromRow, Serialize)]
pub struct StockBuybackInfo {
    pub stock_no: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub price_on_start_date: Option<f64>,
    pub latest_price: Option<Decimal>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct StockBuybackMoreInfo {
    pub stock_no: String,
    pub stock_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub price_on_start_date: Option<f64>,
    pub latest_price: Option<Decimal>,
    pub diff: Option<Decimal>,
    pub diff_percent: Option<Decimal>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum StartPriceFilter {
    All,
    MissingOnly,
    ExistsOnly,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StockExRight {
    pub stock_no: String,
    pub ex_date: NaiveDate,
    pub close_before: f64,
    pub cash_div: f64,
    pub stock_rate: f64,
}

#[derive(Debug, FromRow, Serialize)]
pub struct StockBuybackPeriod {
    pub stock_no: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
