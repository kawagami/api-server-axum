use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct StockRequest {
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct StockChange {
    pub id: i32,
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
    pub status: Option<String>,
    pub stock_name: Option<String>,
    pub start_price: Option<f64>,
    pub end_price: Option<f64>,
    pub change: Option<f64>,
}

#[derive(Serialize, Deserialize, FromRow, Default)]
pub struct StockChangeWithoutId {
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
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
    pub price_diff: f64,     // end - start
    pub percent_change: f64, // %
    pub is_increase: bool,
    pub day_span: i64, // 天數 (可正可負)
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

/// 給 repository 的 fn get_active_buyback_prices 接收 DB 資料用的結構
///
/// 包含股票代號、庫藏股起訖日期，以及起始日價格與最新價格等資訊
#[derive(Debug, FromRow, Serialize)]
pub struct StockBuybackInfo {
    /// 股票代號。
    pub stock_no: String,

    /// 庫藏股開始日期。
    pub start_date: NaiveDate,

    /// 庫藏股結束日期。
    pub end_date: NaiveDate,

    /// 庫藏股開始當日的股價。
    ///
    /// 若資料缺漏則為 `None`。
    pub price_on_start_date: Option<f64>,

    /// 最新的股價。
    ///
    /// 若資料尚未更新則為 `None`。
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

/// 定義查詢篩選條件的枚舉
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum StartPriceFilter {
    /// 全部（不過濾）
    All,
    /// 只有沒起始價格的
    MissingOnly,
    /// 只有有起始價格的
    ExistsOnly,
}

/// 給 repository 的 fn get_stock_buyback_periods 接收 DB 資料用的結構
#[derive(Debug, FromRow, Serialize)]
pub struct StockBuybackPeriod {
    /// 股票代號。
    pub stock_no: String,
    /// 庫藏股開始日期。
    pub start_date: NaiveDate,
    /// 庫藏股結束日期。
    pub end_date: NaiveDate,
}
