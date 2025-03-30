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
    pub stock_no: String,
    pub start_date: String,
    pub end_date: String,
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
