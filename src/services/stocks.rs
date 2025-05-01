use std::collections::HashMap;

use crate::{
    errors::{AppError, RequestError},
    repositories::stocks::{get_stock_closing_prices_by_date_range, upsert_stock_closing_prices},
    state::AppStateV2,
    structs::stocks::{NewStockClosingPrice, StockDayAvgResponse, StockRequest},
    utils::reqwest::{get_json_data, get_raw_html_string},
};
use chrono::{Duration, NaiveDate};
use reqwest::Client;
use scraper::{Html, Selector};

/// Parses HTML document to extract stock buyback information
///
/// # Arguments
/// * `html` - HTML content as a string
///
/// # Returns
/// A vector of StockRequest objects containing extracted stock information
pub fn parse_buyback_stock_raw_html(html: String) -> Vec<StockRequest> {
    let document = Html::parse_document(&html);

    // Define all selectors outside the loop
    let row_selector = Selector::parse("tr.odd, tr.even").unwrap_or_else(|e| {
        tracing::error!("Failed to parse row selector: {}", e);
        Selector::parse("tr").unwrap() // Fallback selector
    });

    let cell_selector = Selector::parse("td").unwrap_or_else(|e| {
        tracing::error!("Failed to parse cell selector: {}", e);
        Selector::parse("td").unwrap() // Should never fail
    });

    // Extract data from each row
    document
        .select(&row_selector)
        .filter_map(|row| {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            // Skip rows that don't have enough cells
            if cells.len() < 11 {
                return None;
            }

            // Extract required data, with better text handling
            let get_cell_text = |index: usize| -> String {
                cells
                    .get(index)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            };

            let stock_no = get_cell_text(1);
            let start_date = get_cell_text(9).replace("/", "");
            let end_date = get_cell_text(10).replace("/", "");

            // Skip records with missing data
            if stock_no.is_empty() || start_date.is_empty() || end_date.is_empty() {
                return None;
            }

            Some(StockRequest {
                stock_no,
                start_date,
                end_date,
            })
        })
        .collect()
}

/// 取得庫藏股列表頁面資訊 string
pub async fn get_buyback_stock_raw_html_string(
    reqwest_client: &Client,
    start_date: &str,
    end_date: &str,
) -> Result<String, AppError> {
    let form_data_pairs = vec![
        ("encodeURIComponent", "1"),
        ("step", "1"),
        ("firstin", "1"),
        ("off", "1"),
        ("TYPEK", "sii"),
        ("d1", start_date),
        ("d2", end_date),
        ("RD", "1"),
    ];

    get_raw_html_string(
        reqwest_client,
        "https://mopsov.twse.com.tw/mops/web/ajax_t35sc09",
        reqwest::Method::POST,
        None,
        Some(form_data_pairs),
    )
    .await
}

/// 取 twse 歷史收盤價
pub async fn get_stock_day_avg(
    request_client: &Client,
    stock_no: &str,
    date: &str, // 格式: YYYYMMDD, 例如 20250101
) -> Result<StockDayAvgResponse, AppError> {
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/STOCK_DAY_AVG?date={}&stockNo={}&response=json&_={}",
        date,
        stock_no,
        get_timestamp()  // 獲取當前時間戳
    );

    // 添加一些常見的 HTTP 標頭，模擬瀏覽器行為
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string());
    headers.insert(
        "Accept".to_string(),
        "application/json, text/javascript, */*; q=0.01".to_string(),
    );
    headers.insert(
        "Accept-Language".to_string(),
        "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7".to_string(),
    );
    headers.insert(
        "Referer".to_string(),
        "https://www.twse.com.tw/".to_string(),
    );

    // 使用我們之前實現的通用 JSON 獲取函數
    get_json_data::<StockDayAvgResponse>(
        request_client,
        &url,
        reqwest::Method::GET,
        Some(headers),
        None, // 不需要表單數據，這是 GET 請求
        None, // 不需要 JSON 請求體
    )
    .await
}

/// 生成當前時間戳，用於 API 請求
fn get_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    now.as_millis().to_string()
}

pub fn parse_stock_day_avg_response(
    api: StockDayAvgResponse,
    stock_no: &str,
) -> Vec<NewStockClosingPrice> {
    api.data
        .iter()
        .filter_map(|row| {
            if row.len() != 2 {
                return None;
            }

            let date_str = row[0].trim();
            let price_str = row[1].trim().replace(",", "");

            // 濾掉非日期資料（像「月平均收盤價」）
            let date_parts: Vec<_> = date_str.split('/').collect();
            if date_parts.len() != 3 {
                return None;
            }

            let year = 1911 + date_parts[0].parse::<i32>().ok()?;
            let month = date_parts[1].parse::<u32>().ok()?;
            let day = date_parts[2].parse::<u32>().ok()?;

            let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
            let close_price = price_str.parse::<f64>().ok()?;

            Some(NewStockClosingPrice {
                stock_no: stock_no.to_string(),
                date,
                close_price,
            })
        })
        .collect()
}

/// 根據指定日期從股票資料中取得單日資料
/// 優先順序：指定日期 > 小於指定日期的最近一天 > 大於指定日期的最近一天
pub fn get_stock_price_by_date(
    stock_prices: &Vec<NewStockClosingPrice>,
    target_date_str: &str,
) -> Result<NewStockClosingPrice, AppError> {
    let target_date = NaiveDate::parse_from_str(
        &format!(
            "{}-{}-{}",
            &target_date_str[0..4],
            &target_date_str[4..6],
            &target_date_str[6..8]
        ),
        "%Y-%m-%d",
    )
    .map_err(|_| {
        AppError::RequestError(RequestError::InvalidDateFormat(target_date_str.to_string()))
    })?;

    if let Some(price) = stock_prices.iter().find(|price| price.date == target_date) {
        return Ok(price.clone());
    }

    let less_than_target = stock_prices
        .iter()
        .filter(|price| price.date < target_date)
        .max_by_key(|price| price.date);

    let greater_than_target = stock_prices
        .iter()
        .filter(|price| price.date > target_date)
        .min_by_key(|price| price.date);

    if let Some(price) = less_than_target {
        Ok(price.clone())
    } else if let Some(price) = greater_than_target {
        Ok(price.clone())
    } else {
        Err(AppError::RequestError(RequestError::StockPriceNotFound))
    }
}

/// 先查詢資料庫有沒有資料 沒有的話才會打外部 API 查詢
/// 依照 指定時間點 > 小於指定時間點 > 大於指定時間點 的優先度取資料
pub async fn fetch_stock_price_for_date(
    state: &AppStateV2,
    stock_no: &str,
    date: &str,
) -> Result<NewStockClosingPrice, AppError> {
    // 抓取資料庫中前後 3 天的範圍
    let date_obj = NaiveDate::parse_from_str(date, "%Y%m%d")?;
    let start_date = (date_obj - Duration::days(3)).format("%Y%m%d").to_string();
    let end_date = (date_obj + Duration::days(3)).format("%Y%m%d").to_string();

    // 從資料庫獲取日期範圍內的所有股票價格
    let db_prices =
        get_stock_closing_prices_by_date_range(state, stock_no, &start_date, &end_date).await?;

    // 資料集合不是空的話 按照優先順序選擇
    if !db_prices.is_empty() {
        // 嘗試從資料集合中按優先順序找出合適的價格
        if let Ok(price) = get_stock_price_by_date(&db_prices, date) {
            // tracing::info!("從資料庫中找到適合的資料 {:#?}", price);
            return Ok(price);
        }
    }

    // 如果沒有合適的資料，從外部 API 獲取
    let response = get_stock_day_avg(state.get_http_client(), stock_no, date).await?;

    // 解析響應
    let closing_prices = parse_stock_day_avg_response(response, stock_no);

    // 將取得的盤後價資料紀錄進資料庫
    upsert_stock_closing_prices(state, &closing_prices).await?;

    // 再次按照優先順序獲取日期的價格
    get_stock_price_by_date(&closing_prices, date)
}
