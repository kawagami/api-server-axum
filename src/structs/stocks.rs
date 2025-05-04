use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize};
use sqlx::prelude::FromRow;

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
                Some(input_date) => {
                    let today = chrono::Local::now().date_naive();
                    if input_date > today {
                        return Err(E::custom(format!(
                            "不允許輸入未來日期: {}, 今天是: {}",
                            value,
                            format!(
                                "{}{:02}{:02}",
                                today.format("%Y").to_string().parse::<i32>().unwrap() - 1911,
                                today.format("%m").to_string(),
                                today.format("%d").to_string()
                            )
                        )));
                    }

                    Ok(value.to_string())
                }
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
