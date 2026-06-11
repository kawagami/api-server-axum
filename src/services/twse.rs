//! TWSE API 共用存取層 — headers、欄位解析、全域併發限制。
//! 所有 www.twse.com.tw 請求一律經過 `fetch_json`（semaphore = 1）避免被 rate limit。

use crate::{errors::AppError, structs::stocks::StockDayAvgResponse, utils::reqwest::get_json_data};
use chrono::NaiveDate;
use reqwest::{Client, Method};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Semaphore;

static TWSE_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

/// 通用 TWSE JSON 回應（stat + 二維字串表格）
#[derive(Deserialize)]
pub struct TwseResponse {
    pub stat: String,
    pub data: Option<Vec<Vec<String>>>,
}

fn headers() -> HashMap<String, String> {
    let mut h = HashMap::new();
    h.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".into());
    h.insert("Accept".into(), "application/json, text/javascript, */*; q=0.01".into());
    h.insert("Accept-Language".into(), "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7".into());
    h.insert("Referer".into(), "https://www.twse.com.tw/".into());
    h
}

/// TWSE 數值欄位解析 — 處理千分位逗號與 "--"/"-" 空值
pub fn parse_f64(s: &str) -> Option<f64> {
    let clean = s.trim().replace(",", "");
    if clean.is_empty() || clean == "--" || clean == "-" {
        return None;
    }
    clean.parse().ok()
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &Client,
    url: &str,
) -> Result<T, AppError> {
    let _permit = TWSE_SEMAPHORE.acquire().await.expect("semaphore closed");
    get_json_data(client, url, Method::GET, Some(headers()), None, None).await
}

/// 月成交資訊（STOCK_DAY）— month 取該月任一日
pub async fn fetch_stock_day(
    client: &Client,
    stock_code: &str,
    month: NaiveDate,
) -> Result<TwseResponse, AppError> {
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/STOCK_DAY?date={}&stockNo={}&response=json",
        month.format("%Y%m01"),
        stock_code
    );
    fetch_json(client, &url).await
}

/// 月平均收盤價（STOCK_DAY_AVG）
pub async fn fetch_stock_day_avg(
    client: &Client,
    stock_no: &str,
    date: NaiveDate,
) -> Result<StockDayAvgResponse, AppError> {
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/STOCK_DAY_AVG?date={}&stockNo={}&response=json&_={}",
        date.format("%Y%m%d"),
        stock_no,
        timestamp_millis()
    );
    fetch_json(client, &url).await
}

/// 除權除息（TWT49U）
pub async fn fetch_ex_rights(
    client: &Client,
    start: &str,
    end: &str,
) -> Result<TwseResponse, AppError> {
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/exRight/TWT49U?startDate={}&endDate={}&response=json",
        start, end
    );
    fetch_json(client, &url).await
}

fn timestamp_millis() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string()
}
