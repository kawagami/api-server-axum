//! TWSE API 共用存取層 — headers、欄位解析、全域併發限制。
//! 所有 www.twse.com.tw 請求一律經過 `fetch_json`（semaphore = 1）避免被 rate limit。

use crate::{
    errors::AppError,
    structs::stocks::StockDayAvgResponse,
    utils::reqwest::{get_json_data, get_raw_html_string},
};
use chrono::NaiveDate;
use reqwest::{Client, Method};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Semaphore;

static TWSE_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

/// 通用 TWSE JSON 回應（stat + 欄名 + 二維字串表格）
#[derive(Deserialize)]
pub struct TwseResponse {
    pub stat: String,
    /// 欄名。解析時依名稱找索引，TWSE 改欄位順序才會被發現，而不是靜默讀錯欄
    #[serde(default)]
    pub fields: Vec<String>,
    pub data: Option<Vec<Vec<String>>>,
}

impl TwseResponse {
    /// 依欄名取索引；任一欄不存在回 None（= 上游改版，呼叫端要當成失敗而非「沒資料」）
    pub fn field_indices<const N: usize>(&self, names: [&str; N]) -> Option<[usize; N]> {
        let mut out = [0; N];
        for (slot, name) in out.iter_mut().zip(names) {
            *slot = self.fields.iter().position(|f| f.trim() == name)?;
        }
        Some(out)
    }

    /// TWSE 查無資料時 stat 不是 "OK" 而是這句話 —— 它是正常的「確認過沒有」，不是錯誤
    pub fn is_no_data(&self) -> bool {
        self.stat.contains("沒有符合條件的資料")
    }
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

/// 取得 CSV / 純文字回應（沿用 TWSE headers + 全域 semaphore）
pub async fn fetch_text(client: &Client, url: &str) -> Result<String, AppError> {
    let _permit = TWSE_SEMAPHORE.acquire().await.expect("semaphore closed");
    get_raw_html_string(client, url, Method::GET, Some(headers()), None).await
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
