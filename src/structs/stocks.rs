use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ClosingPrice")]
    pub closing_price: String,
    #[serde(rename = "MonthlyAveragePrice")]
    pub monthly_average_price: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct StockRequest {
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Default)]
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

#[derive(Serialize, Deserialize, sqlx::FromRow, Default)]
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
pub struct BuybackDuration {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockChangeId {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct Conditions {
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StockDayAvgResponse {
    pub stat: String,
    pub title: String,
    pub fields: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub notes: Vec<String>,
}

#[derive(Deserialize)]
pub struct GetStockHistoryPriceRequest {
    pub stock_no: String,
    pub date: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct StockClosingPrice {
    pub id: i32,
    pub stock_no: String,
    pub date: chrono::NaiveDate,
    pub close_price: f64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Serialize, Clone)]
pub struct NewStockClosingPrice {
    pub stock_no: String,
    pub date: chrono::NaiveDate,
    pub close_price: f64,
}
