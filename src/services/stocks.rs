use crate::{
    errors::{AppError, RequestError},
    repositories::stocks::{
        get_stock_closing_prices_by_date_range, insert_stock_day_all_batch,
        upsert_stock_closing_prices,
    },
    repositories::stocks as stocks_repo,
    state::AppState,
    structs::stocks::{
        Conditions, GetStockDayAll, NewStockClosingPrice, Pagination, StockBuybackMoreInfo,
        StockBuybackPeriod, StockChangePaginatedResponse, StockChangeWithoutId, StockClosingPriceResponse,
        StockDayAll, StockDayAllInsertRow, StockDayAvgResponse, StockRequest, StockStats, TwseApiResponse,
    },
    utils::reqwest::{get_json_data, get_raw_html_string},
};
use chrono::{Duration, NaiveDate};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

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
    )?;

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
        Err(RequestError::NotFound.into())
    }
}

/// 先查詢資料庫有沒有資料 沒有的話才會打外部 API 查詢
/// 依照 指定時間點 > 小於指定時間點 > 大於指定時間點 的優先度取資料
pub async fn fetch_stock_price_for_date(
    state: &AppState,
    stock_no: &str,
    date: &str,
) -> Result<NewStockClosingPrice, AppError> {
    // 檢查是否為未來日期
    let date_obj = NaiveDate::parse_from_str(date, "%Y%m%d")?;
    let today = chrono::Local::now().date_naive();
    if date_obj > today {
        return Err(RequestError::InvalidContent(format!(
            "Cannot fetch stock price for future date: {}",
            date
        ))
        .into());
    }

    // 抓取資料庫中前後 3 天的範圍
    let start_date = (date_obj - Duration::days(3)).format("%Y%m%d").to_string();
    let end_date = (date_obj + Duration::days(3)).format("%Y%m%d").to_string();

    // 從資料庫獲取日期範圍內的所有股票價格
    let db_prices =
        get_stock_closing_prices_by_date_range(state, stock_no, &start_date, &end_date).await?;

    // 資料集合不是空的話 按照優先順序選擇
    if !db_prices.is_empty() {
        // 嘗試從資料集合中按優先順序找出合適的價格
        if let Ok(price) = get_stock_price_by_date(&db_prices, date) {
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

// 工具函數：四捨五入到小數點 N 位
pub fn round_to_n_decimal(value: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
}

fn roc_to_ad(roc_date: &str) -> Result<String, AppError> {
    if roc_date.len() != 7 {
        return Err(RequestError::InvalidContent(format!("invalid ROC date: {}", roc_date)).into());
    }
    let roc_year: u32 = roc_date[..3]
        .parse()
        .map_err(|_| RequestError::InvalidContent(format!("invalid ROC date: {}", roc_date)))?;
    Ok(format!("{}{}", roc_year + 1911, &roc_date[3..]))
}

fn extract_stock_name(title: &str, stock_no: &str) -> String {
    let marker = format!("{} ", stock_no);
    title
        .find(&marker)
        .and_then(|pos| {
            let after = &title[pos + marker.len()..];
            after.split_whitespace().next().map(str::to_string)
        })
        .unwrap_or_else(|| "未知公司".to_string())
}

pub async fn get_stock_change_info(
    state: &AppState,
    stock_form: &StockRequest,
) -> Result<StockChangeWithoutId, AppError> {
    let start_date_ad = roc_to_ad(&stock_form.start_date)?;
    let end_date_ad = roc_to_ad(&stock_form.end_date)?;

    let (start_response, end_price_data) = tokio::try_join!(
        get_stock_day_avg(state.get_http_client(), &stock_form.stock_no, &start_date_ad),
        fetch_stock_price_for_date(state, &stock_form.stock_no, &end_date_ad)
    )?;

    let stock_name = extract_stock_name(&start_response.title, &stock_form.stock_no);
    let start_prices = parse_stock_day_avg_response(start_response, &stock_form.stock_no);

    if start_prices.is_empty() {
        return Err(RequestError::InvalidContent(format!(
            "no price data for stock {} on {}",
            stock_form.stock_no, stock_form.start_date
        ))
        .into());
    }

    upsert_stock_closing_prices(state, &start_prices).await?;
    let start_price_data = get_stock_price_by_date(&start_prices, &start_date_ad)?;

    let start_price = round_to_n_decimal(start_price_data.close_price, 2);
    let end_price = round_to_n_decimal(end_price_data.close_price, 2);
    let change = round_to_n_decimal((end_price - start_price) / start_price * 100.0, 2);

    Ok(StockChangeWithoutId {
        stock_no: stock_form.stock_no.clone(),
        stock_name: Some(stock_name),
        start_date: stock_form.start_date.clone(),
        start_price: Some(start_price),
        end_date: stock_form.end_date.clone(),
        end_price: Some(end_price),
        change: Some(change),
        status: None,
    })
}

pub async fn stock_day_all_service(state: &AppState) -> Result<(), AppError> {
    let url = "https://www.twse.com.tw/exchangeReport/STOCK_DAY_ALL";
    let resp: TwseApiResponse = state
        .get_http_client()
        .get(url)
        .send()
        .await?
        .json()
        .await?;

    let trade_date = chrono::NaiveDate::parse_from_str(&resp.date, "%Y%m%d")?;

    let parse_i64 = |s: &str| -> Option<i64> {
        if s.is_empty() || s == "--" { None } else { s.trim().replace(",", "").parse().ok() }
    };

    let parse_f64 = |s: &str| -> Option<f64> {
        if s.is_empty() || s == "--" { None } else { s.trim().replace(",", "").parse().ok() }
    };

    let rows: Vec<StockDayAllInsertRow> = resp.data.iter()
        .filter(|row| row.len() >= 10)
        .filter_map(|row| {
            Some(StockDayAllInsertRow {
                trade_date,
                stock_code: row[0].clone(),
                stock_name: row[1].clone(),
                trade_volume: parse_i64(&row[2])?,
                trade_amount: parse_i64(&row[3])?,
                open_price: parse_f64(&row[4])?,
                high_price: parse_f64(&row[5])?,
                low_price: parse_f64(&row[6])?,
                close_price: parse_f64(&row[7])?,
                price_change: parse_f64(&row[8])?,
                transaction_count: parse_i64(&row[9]).unwrap_or(0) as i32,
            })
        })
        .collect();

    insert_stock_day_all_batch(state, &rows).await
}

pub async fn get_all_stock_changes(
    state: &AppState,
    conditions: Conditions,
) -> Result<StockChangePaginatedResponse, AppError> {
    stocks_repo::get_all_stock_changes(state, conditions).await
}

pub async fn update_one_stock_change_pending(state: &AppState, id: i32) -> Result<(), AppError> {
    stocks_repo::update_one_stock_change_pending(state, id).await
}

pub async fn get_stock_day_all_list(
    state: &AppState,
    params: GetStockDayAll,
    pagination: Pagination,
) -> Result<Vec<StockDayAll>, AppError> {
    stocks_repo::get_stock_day_all(
        state,
        params.stock_code,
        params.trade_date,
        pagination.limit,
        pagination.offset,
    )
    .await
}

pub async fn get_active_buyback_prices(
    state: &AppState,
) -> Result<Vec<StockBuybackMoreInfo>, AppError> {
    stocks_repo::get_active_buyback_prices(state).await
}

pub async fn get_stock_buyback_periods(
    state: &AppState,
) -> Result<Vec<StockBuybackPeriod>, AppError> {
    stocks_repo::get_stock_buyback_periods(state).await
}

pub async fn get_closing_price_pair_stats(
    state: &AppState,
    payload: &StockRequest,
) -> Result<StockClosingPriceResponse, AppError> {
    let (start_price, end_price) = tokio::try_join!(
        fetch_stock_price_for_date(state, &payload.stock_no, &payload.start_date),
        fetch_stock_price_for_date(state, &payload.stock_no, &payload.end_date)
    )?;

    let price_diff = round_to_n_decimal(end_price.close_price - start_price.close_price, 2);
    let raw_percent_change = if start_price.close_price != 0.0 {
        (price_diff / start_price.close_price) * 100.0
    } else {
        0.0
    };
    let percent_change = round_to_n_decimal(raw_percent_change, 2);
    let is_increase = price_diff > 0.0;
    let day_span = (end_price.date - start_price.date).num_days();

    Ok(StockClosingPriceResponse {
        prices: (start_price, end_price),
        stats: StockStats {
            price_diff,
            percent_change,
            is_increase,
            day_span,
        },
    })
}
