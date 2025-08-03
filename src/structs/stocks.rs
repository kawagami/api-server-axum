use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer, Serialize};
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

// 實現ROC日期的自定義反序列化
fn deserialize_roc_date<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct ROCDateVisitor;

    impl<'de> de::Visitor<'de> for ROCDateVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("民國日期格式的字符串 (格式: YYYMMDD，例如: 1140504)")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            // 檢查格式: 需要7位數字
            let re = Regex::new(r"^(\d{3})(\d{2})(\d{2})$").unwrap();
            if !re.is_match(value) {
                return Err(E::custom(format!(
                    "日期格式必須為民國年數三位數+月份兩位數+日期兩位數 (例如: 1140504), 收到: {}",
                    value
                )));
            }

            let caps = re.captures(value).unwrap();
            let roc_year: i32 = caps.get(1).unwrap().as_str().parse().unwrap();
            let month: u32 = caps.get(2).unwrap().as_str().parse().unwrap();
            let day: u32 = caps.get(3).unwrap().as_str().parse().unwrap();

            // 將民國年轉換為西元年
            let gregorian_year = roc_year + 1911;

            // 檢查日期是否有效
            match NaiveDate::from_ymd_opt(gregorian_year, month, day) {
                Some(_) => Ok(value.to_string()),
                None => Err(E::custom(format!("無效的日期: {}", value))),
            }
        }
    }

    deserializer.deserialize_str(ROCDateVisitor)
}

// 修改您的結構體，使用自定義反序列化
#[derive(Debug, Serialize, Deserialize)]
pub struct BuybackDuration {
    #[serde(deserialize_with = "deserialize_roc_date")]
    pub start_date: String,
    #[serde(deserialize_with = "deserialize_roc_date")]
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

#[derive(Serialize, FromRow)]
pub struct StockClosingPrice {
    pub id: i32,
    pub stock_no: String,
    pub date: NaiveDate,
    pub close_price: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
