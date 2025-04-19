use std::collections::HashMap;

use crate::{
    errors::AppError,
    structs::stocks::{StockDayAvgResponse, StockRequest},
    utils::reqwest::{get_json_data, get_raw_html_string},
};
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
