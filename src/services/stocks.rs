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
        StockBuybackPeriod, StockChange, StockChangePaginatedResponse, StockChangeRef,
        StockClosingPriceResponse, StockDayAll, StockDayAllInsertRow, StockDayAvgResponse,
        BuybackRecord, StockRequest, StockStats, TwseApiResponse,
    },
    utils::reqwest::{get_json_data, get_raw_html_string},
};
use chrono::{Duration, NaiveDate};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

fn parse_roc_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i32>().ok()? + 1911;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

pub fn parse_buyback_stock_raw_html(html: String) -> Vec<BuybackRecord> {
    let document = Html::parse_document(&html);
    let row_selector = Selector::parse("tr.odd, tr.even").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    document
        .select(&row_selector)
        .filter_map(|row| {
            let cells: Vec<_> = row.select(&cell_selector).collect();

            if cells.len() < 11 {
                return None;
            }

            let get_cell_text = |index: usize| -> String {
                cells
                    .get(index)
                    .map(|cell| cell.text().collect::<String>().trim().to_string())
                    .unwrap_or_default()
            };

            let stock_no = get_cell_text(1);
            let start_raw = get_cell_text(9);
            let end_raw = get_cell_text(10);

            let start_date = parse_roc_date(&start_raw);
            let end_date = parse_roc_date(&end_raw);

            if stock_no.is_empty() || start_date.is_none() || end_date.is_none() {
                tracing::warn!(
                    "parse_buyback_stock_raw_html: skipped row stock_no={:?} start={:?} end={:?}",
                    stock_no, start_raw, end_raw
                );
                return None;
            }

            Some(BuybackRecord {
                stock_no,
                start_date: start_date.unwrap(),
                end_date: end_date.unwrap(),
            })
        })
        .collect()
}

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

pub async fn get_stock_day_avg(
    request_client: &Client,
    stock_no: &str,
    date: NaiveDate,
) -> Result<StockDayAvgResponse, AppError> {
    let date_str = date.format("%Y%m%d").to_string();
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/STOCK_DAY_AVG?date={}&stockNo={}&response=json&_={}",
        date_str,
        stock_no,
        get_timestamp()
    );

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

    get_json_data::<StockDayAvgResponse>(
        request_client,
        &url,
        reqwest::Method::GET,
        Some(headers),
        None,
        None,
    )
    .await
}

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

/// 按優先順序取日期：指定日 > 小於指定日最近 > 大於指定日最近
pub fn get_stock_price_by_date(
    stock_prices: &[NewStockClosingPrice],
    target_date: NaiveDate,
) -> Result<NewStockClosingPrice, AppError> {
    if let Some(price) = stock_prices.iter().find(|p| p.date == target_date) {
        return Ok(price.clone());
    }

    let less = stock_prices.iter().filter(|p| p.date < target_date).max_by_key(|p| p.date);
    let greater = stock_prices.iter().filter(|p| p.date > target_date).min_by_key(|p| p.date);

    if let Some(price) = less {
        Ok(price.clone())
    } else if let Some(price) = greater {
        Ok(price.clone())
    } else {
        Err(RequestError::NotFound.into())
    }
}

/// DB 優先，cache miss 才打 TWSE
pub async fn fetch_stock_price_for_date(
    state: &AppState,
    stock_no: &str,
    date: NaiveDate,
) -> Result<NewStockClosingPrice, AppError> {
    let today = chrono::Local::now().date_naive();
    if date > today {
        return Err(RequestError::InvalidContent(format!(
            "Cannot fetch stock price for future date: {}",
            date
        ))
        .into());
    }

    let range_start = date - Duration::days(3);
    let range_end = date + Duration::days(3);

    let db_prices =
        get_stock_closing_prices_by_date_range(state, stock_no, range_start, range_end).await?;

    if !db_prices.is_empty() {
        if let Ok(price) = get_stock_price_by_date(&db_prices, date) {
            return Ok(price);
        }
    }

    let response = get_stock_day_avg(state.get_http_client(), stock_no, date).await?;
    let closing_prices = parse_stock_day_avg_response(response, stock_no);
    upsert_stock_closing_prices(state, &closing_prices).await?;
    get_stock_price_by_date(&closing_prices, date)
}

pub async fn get_stock_change_info(
    state: &AppState,
    stock_ref: &StockChangeRef,
) -> Result<StockChange, AppError> {
    let (start_price_data, end_price_data) = tokio::try_join!(
        fetch_stock_price_for_date(state, &stock_ref.stock_no, stock_ref.start_date),
        fetch_stock_price_for_date(state, &stock_ref.stock_no, stock_ref.end_date)
    )?;

    let stock_name = stocks_repo::get_stock_name_by_code(state, &stock_ref.stock_no)
        .await
        .ok()
        .flatten();

    let start_price = round_to_n_decimal(start_price_data.close_price, 2);
    let end_price = round_to_n_decimal(end_price_data.close_price, 2);
    let change = round_to_n_decimal((end_price - start_price) / start_price * 100.0, 2);

    Ok(StockChange {
        id: None,
        stock_no: stock_ref.stock_no.clone(),
        stock_name,
        start_date: stock_ref.start_date,
        start_price: Some(start_price),
        end_date: stock_ref.end_date,
        end_price: Some(end_price),
        change: Some(change),
        status: None,
    })
}

pub fn round_to_n_decimal(value: f64, decimals: u32) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (value * factor).round() / factor
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
    let start_date = NaiveDate::parse_from_str(&payload.start_date, "%Y%m%d")
        .map_err(|_| RequestError::InvalidContent(format!("invalid start_date: {}", payload.start_date)))?;
    let end_date = NaiveDate::parse_from_str(&payload.end_date, "%Y%m%d")
        .map_err(|_| RequestError::InvalidContent(format!("invalid end_date: {}", payload.end_date)))?;

    let (start_price, end_price) = tokio::try_join!(
        fetch_stock_price_for_date(state, &payload.stock_no, start_date),
        fetch_stock_price_for_date(state, &payload.stock_no, end_date)
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
