use crate::{
    errors::AppError,
    repositories::{
        portfolio as portfolio_repo,
        redis as redis_repo,
        stocks::{find_ex_rights_checked, get_ex_rights_by_range, get_stock_closing_prices_by_date_range, get_stock_name_by_code, upsert_ex_rights, upsert_ex_rights_checked, upsert_stock_closing_prices},
    },
    state::AppState,
    structs::{
        portfolio::{HistoryRecord, PortfolioEntry, PortfolioRequest, PortfolioSummaryEntry},
        stocks::{NewStockClosingPrice, StockExRight},
    },
    utils::reqwest::get_json_data,
};
use chrono::{Datelike, Local, Months, NaiveDate};
use futures::future::try_join_all;
use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

// TWT49U field indices — adjust here if TWSE changes column order
const EX_IDX_CODE: usize = 0;
const EX_IDX_DATE: usize = 2;
const EX_IDX_CLOSE_BEFORE: usize = 3;
const EX_IDX_STOCK_RATE: usize = 4;
const EX_IDX_CASH_DIV: usize = 5;

#[derive(Deserialize)]
struct TwseResponse {
    stat: String,
    data: Option<Vec<Vec<String>>>,
}

struct DayClose {
    date: NaiveDate,
    close: f64,
}

struct ExEvent {
    date: NaiveDate,
    close_before: f64,
    cash_div: f64,
    stock_rate: f64,
}

pub async fn get_by_member(state: &AppState, member_id: i64) -> Result<Vec<PortfolioEntry>, AppError> {
    portfolio_repo::get_by_member(state, member_id).await
}

pub async fn create(
    state: &AppState,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    portfolio_repo::create(state, member_id, req).await
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    portfolio_repo::update(state, id, member_id, req).await
}

pub async fn delete(state: &AppState, id: Uuid, member_id: i64) -> Result<(), AppError> {
    portfolio_repo::delete(state, id, member_id).await
}

pub async fn get_history(
    state: &AppState,
    id: Uuid,
    member_id: i64,
) -> Result<Vec<HistoryRecord>, AppError> {
    let entry = portfolio_repo::get_by_id_for_member(state, id, member_id).await?;
    let today = Local::now().date_naive();

    let closes = fetch_all_closing_prices(state, &entry.stock_code, entry.buy_date, today).await?;
    let ex_events = fetch_ex_events(state, &entry.stock_code, entry.buy_date, today).await?;

    Ok(build_history(entry.cost_per_share, entry.shares, closes, ex_events))
}

pub async fn get_summary(
    state: &AppState,
    member_id: i64,
) -> Result<Vec<PortfolioSummaryEntry>, AppError> {
    let t = Instant::now();
    let entries = portfolio_repo::get_by_member(state, member_id).await?;
    let entry_count = entries.len();
    let today = Local::now().date_naive();

    let result = try_join_all(entries.into_iter().map(|entry| {
        let state = state.clone();
        async move {
            let t_entry = Instant::now();
            let (closes, ex_events) = tokio::try_join!(
                fetch_all_closing_prices(&state, &entry.stock_code, entry.buy_date, today),
                fetch_ex_events(&state, &entry.stock_code, entry.buy_date, today),
            )?;
            tracing::info!("portfolio summary entry {}: {}ms", entry.stock_code, t_entry.elapsed().as_millis());
            let history = build_history(entry.cost_per_share, entry.shares, closes, ex_events);

            let latest = history.last();
            let current_price = latest.map(|r| r.close);
            let current_value = latest.map(|r| r.close * entry.shares as f64);
            let pnl = latest.map(|r| r.pnl);
            let pnl_pct = latest.map(|r| r.pnl_pct);

            let stock_name = get_stock_name_by_code(&state, &entry.stock_code)
                .await
                .unwrap_or(None);

            Ok::<_, AppError>(PortfolioSummaryEntry {
                id: entry.id,
                member_id: entry.member_id,
                stock_code: entry.stock_code,
                stock_name,
                buy_date: entry.buy_date,
                cost_per_share: entry.cost_per_share,
                shares: entry.shares,
                current_price,
                current_value,
                pnl,
                pnl_pct,
                created_at: entry.created_at,
                updated_at: entry.updated_at,
            })
        }
    }))
    .await?;

    tracing::info!("portfolio summary member={}: {} entries, {}ms total", member_id, entry_count, t.elapsed().as_millis());
    Ok(result)
}

fn twse_headers() -> HashMap<String, String> {
    let mut h = HashMap::new();
    h.insert("User-Agent".into(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".into());
    h.insert("Accept".into(), "application/json, text/javascript, */*; q=0.01".into());
    h.insert("Accept-Language".into(), "zh-TW,zh;q=0.9,en-US;q=0.8,en;q=0.7".into());
    h.insert("Referer".into(), "https://www.twse.com.tw/".into());
    h
}

fn parse_roc_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.trim().split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].trim().parse().ok()?;
    let month: u32 = parts[1].trim().parse().ok()?;
    let day: u32 = parts[2].trim().parse().ok()?;
    NaiveDate::from_ymd_opt(year + 1911, month, day)
}

fn parse_twse_f64(s: &str) -> Option<f64> {
    let clean = s.trim().replace(",", "");
    if clean.is_empty() || clean == "--" || clean == "-" {
        return None;
    }
    clean.parse().ok()
}

fn redis_serialize_closes(closes: &[DayClose]) -> Option<String> {
    let v: Vec<(String, f64)> = closes
        .iter()
        .map(|d| (d.date.format("%Y-%m-%d").to_string(), d.close))
        .collect();
    serde_json::to_string(&v).ok()
}

fn redis_deserialize_closes(s: &str) -> Option<Vec<DayClose>> {
    let rows: Vec<(String, f64)> = serde_json::from_str(s).ok()?;
    rows.into_iter()
        .map(|(d, c)| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok().map(|date| DayClose { date, close: c }))
        .collect()
}

async fn fetch_closing_month(
    state: &AppState,
    stock_code: &str,
    month: NaiveDate,
) -> Result<Vec<DayClose>, AppError> {
    let t = Instant::now();
    let month_str = month.format("%Y%m").to_string();
    let cache_key = format!("twse:stock_day:{}:{}", stock_code, month_str);
    let today = Local::now().date_naive();
    let is_current = month.year() == today.year() && month.month() == today.month();
    let ttl = if is_current { 3600u64 } else { 604800u64 };

    // 1. Redis
    if let Ok(Some(cached)) = redis_repo::cache_get(state, &cache_key).await {
        if let Some(data) = redis_deserialize_closes(&cached) {
            tracing::info!("portfolio close {}/{}: redis hit {}ms", stock_code, month_str, t.elapsed().as_millis());
            return Ok(data);
        }
    }

    // 2. DB — past months only (historical data is complete; current month may be partial)
    if !is_current {
        let first_day = month.format("%Y%m%d").to_string();
        let last_day = month
            .checked_add_months(Months::new(1))
            .and_then(|d| d.pred_opt())
            .map(|d| d.format("%Y%m%d").to_string())
            .unwrap_or_default();

        let db_rows = get_stock_closing_prices_by_date_range(state, stock_code, &first_day, &last_day).await?;
        if !db_rows.is_empty() {
            let closes: Vec<DayClose> = db_rows.iter().map(|r| DayClose { date: r.date, close: r.close_price }).collect();
            if let Some(json) = redis_serialize_closes(&closes) {
                let _ = redis_repo::cache_set(state, &cache_key, &json, ttl).await;
            }
            tracing::info!("portfolio close {}/{}: db hit {}ms", stock_code, month_str, t.elapsed().as_millis());
            return Ok(closes);
        }
    }

    // 3. TWSE
    let date_str = month.format("%Y%m01").to_string();
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/afterTrading/STOCK_DAY?date={}&stockNo={}&response=json",
        date_str, stock_code
    );

    let resp: TwseResponse = match get_json_data(
        state.get_http_client(),
        &url,
        Method::GET,
        Some(twse_headers()),
        None,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("TWSE STOCK_DAY fetch failed {}/{}: {}", stock_code, date_str, e);
            return Ok(vec![]);
        }
    };

    let closes: Vec<DayClose> = if resp.stat == "OK" {
        resp.data
            .unwrap_or_default()
            .iter()
            .filter_map(|row| {
                if row.len() < 7 { return None; }
                let date = parse_roc_date(&row[0])?;
                let close = parse_twse_f64(&row[6])?;
                Some(DayClose { date, close })
            })
            .collect()
    } else {
        vec![]
    };

    tracing::info!("portfolio close {}/{}: twse hit {}ms ({} rows)", stock_code, month_str, t.elapsed().as_millis(), closes.len());

    // 4. Write DB
    if !closes.is_empty() {
        let prices: Vec<NewStockClosingPrice> = closes
            .iter()
            .map(|d| NewStockClosingPrice { stock_no: stock_code.to_string(), date: d.date, close_price: d.close })
            .collect();
        if let Err(e) = upsert_stock_closing_prices(state, &prices).await {
            tracing::warn!("upsert_stock_closing_prices failed {}: {}", stock_code, e);
        }
    }

    // 5. Write Redis
    if let Some(json) = redis_serialize_closes(&closes) {
        let _ = redis_repo::cache_set(state, &cache_key, &json, ttl).await;
    }

    Ok(closes)
}

async fn fetch_all_closing_prices(
    state: &AppState,
    stock_code: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<DayClose>, AppError> {
    let t = Instant::now();
    let mut all: Vec<DayClose> = Vec::new();
    let mut current = NaiveDate::from_ymd_opt(from.year(), from.month(), 1).unwrap();
    let end_month = NaiveDate::from_ymd_opt(to.year(), to.month(), 1).unwrap();
    let mut month_count = 0usize;
    while current <= end_month {
        let mut month_data = fetch_closing_month(state, stock_code, current).await?;
        all.append(&mut month_data);
        current = current.checked_add_months(Months::new(1)).unwrap();
        month_count += 1;
    }
    all.retain(|d| d.date >= from);
    all.sort_by_key(|d| d.date);

    tracing::info!("portfolio closes {}: {} months, {} days, {}ms total", stock_code, month_count, all.len(), t.elapsed().as_millis());
    Ok(all)
}

async fn fetch_ex_events(
    state: &AppState,
    stock_code: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<ExEvent>, AppError> {
    let t = Instant::now();
    let start_str = from.format("%Y%m%d").to_string();
    let end_str = to.format("%Y%m%d").to_string();
    let cache_key = format!("twse:exright:{}:{}", stock_code, start_str);

    // 1. Redis
    if let Ok(Some(cached)) = redis_repo::cache_get(state, &cache_key).await {
        if let Ok(rows) = serde_json::from_str::<Vec<(String, f64, f64, f64)>>(&cached) {
            let events: Vec<ExEvent> = rows
                .into_iter()
                .filter_map(|(d, cb, cd, sr)| {
                    NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok().map(|date| ExEvent {
                        date,
                        close_before: cb,
                        cash_div: cd,
                        stock_rate: sr,
                    })
                })
                .collect();
            tracing::info!("portfolio exright {}: redis hit {}ms ({} events)", stock_code, t.elapsed().as_millis(), events.len());
            return Ok(events);
        }
    }

    // 2. DB (ex-rights rows)
    let db_rows = get_ex_rights_by_range(state, stock_code, from, to).await?;
    if !db_rows.is_empty() {
        let events: Vec<ExEvent> = db_rows
            .iter()
            .map(|r| ExEvent { date: r.ex_date, close_before: r.close_before, cash_div: r.cash_div, stock_rate: r.stock_rate })
            .collect();
        tracing::info!("portfolio exright {}: db hit {}ms ({} events)", stock_code, t.elapsed().as_millis(), events.len());
        cache_ex_events(state, &cache_key, &events).await;
        return Ok(events);
    }

    // 2.5. DB (checked table) — confirmed no ex-rights within 30 days
    if let Ok(Some(checked_at)) = find_ex_rights_checked(state, stock_code, from).await {
        let age_days = (chrono::Utc::now() - checked_at).num_days();
        if age_days < 30 {
            tracing::info!("portfolio exright {}: checked hit {}ms (0 events, age={}d)", stock_code, t.elapsed().as_millis(), age_days);
            cache_ex_events(state, &cache_key, &[]).await;
            return Ok(vec![]);
        }
    }

    // 3. TWSE
    let url = format!(
        "https://www.twse.com.tw/rwd/zh/exRight/TWT49U?startDate={}&endDate={}&response=json",
        start_str, end_str
    );

    let resp: TwseResponse = match get_json_data(
        state.get_http_client(),
        &url,
        Method::GET,
        Some(twse_headers()),
        None,
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("TWSE TWT49U fetch failed {}/{}-{}: {}", stock_code, start_str, end_str, e);
            return Ok(vec![]);
        }
    };

    let events: Vec<ExEvent> = if resp.stat == "OK" {
        resp.data
            .unwrap_or_default()
            .iter()
            .filter_map(|row| {
                let min_len = EX_IDX_CASH_DIV + 1;
                if row.len() < min_len { return None; }
                if row[EX_IDX_CODE].trim() != stock_code { return None; }
                let date = parse_roc_date(&row[EX_IDX_DATE])?;
                let close_before = parse_twse_f64(&row[EX_IDX_CLOSE_BEFORE]).unwrap_or(0.0);
                let stock_rate = parse_twse_f64(&row[EX_IDX_STOCK_RATE]).unwrap_or(0.0);
                let cash_div = parse_twse_f64(&row[EX_IDX_CASH_DIV]).unwrap_or(0.0);
                Some(ExEvent { date, close_before, cash_div, stock_rate })
            })
            .collect()
    } else {
        vec![]
    };

    tracing::info!("portfolio exright {}: twse hit {}ms ({} events)", stock_code, t.elapsed().as_millis(), events.len());

    // 4. Write DB
    if !events.is_empty() {
        let rows: Vec<StockExRight> = events
            .iter()
            .map(|e| StockExRight { stock_no: stock_code.to_string(), ex_date: e.date, close_before: e.close_before, cash_div: e.cash_div, stock_rate: e.stock_rate })
            .collect();
        if let Err(e) = upsert_ex_rights(state, &rows).await {
            tracing::warn!("upsert_ex_rights failed {}: {}", stock_code, e);
        }
    }

    // 4.5. Write checked record (regardless of result, marks TWSE was queried)
    if let Err(e) = upsert_ex_rights_checked(state, stock_code, from).await {
        tracing::warn!("upsert_ex_rights_checked failed {}: {}", stock_code, e);
    }

    // 5. Write Redis
    cache_ex_events(state, &cache_key, &events).await;

    Ok(events)
}

async fn cache_ex_events(state: &AppState, key: &str, events: &[ExEvent]) {
    let v: Vec<(String, f64, f64, f64)> = events
        .iter()
        .map(|e| (e.date.format("%Y-%m-%d").to_string(), e.close_before, e.cash_div, e.stock_rate))
        .collect();
    if let Ok(json) = serde_json::to_string(&v) {
        let _ = redis_repo::cache_set(state, key, &json, 86400).await;
    }
}

fn build_history(
    cost: f64,
    shares: i64,
    closes: Vec<DayClose>,
    mut ex_events: Vec<ExEvent>,
) -> Vec<HistoryRecord> {
    ex_events.sort_by_key(|e| e.date);

    let mut adjusted_cost = cost;
    let mut applied = 0usize;
    let mut records = Vec::with_capacity(closes.len());

    for day in &closes {
        while applied < ex_events.len() && ex_events[applied].date <= day.date {
            let ev = &ex_events[applied];
            if ev.close_before > 0.0 {
                let numer = ev.close_before - ev.cash_div;
                let denom = ev.close_before * (1.0 + ev.stock_rate / 1000.0);
                if denom > 0.0 {
                    adjusted_cost = adjusted_cost * numer / denom;
                }
            }
            applied += 1;
        }

        let pnl = (day.close - adjusted_cost) * shares as f64;
        let pnl_pct = if adjusted_cost != 0.0 {
            (day.close - adjusted_cost) / adjusted_cost * 100.0
        } else {
            0.0
        };

        records.push(HistoryRecord {
            date: day.date,
            close: day.close,
            adjusted_cost,
            pnl,
            pnl_pct,
        });
    }

    records
}
