use crate::{
    errors::AppError,
    repositories::{
        portfolio as portfolio_repo,
        redis as redis_repo,
        stocks::{find_ex_rights_checked, get_ex_rights_by_range, get_stock_closing_prices_by_date_range, get_stock_name_by_code, upsert_ex_rights, upsert_ex_rights_checked, upsert_stock_closing_prices},
    },
    structs::{
        portfolio::{HistoryRecord, PortfolioEntry, PortfolioRequest, PortfolioSummaryEntry},
        stocks::{NewStockClosingPrice, StockExRight},
    },
    utils::date::parse_roc_date,
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use chrono::{Datelike, Months, NaiveDate};
use futures::future::try_join_all;
use reqwest::Client;
use sqlx::{Pool, Postgres};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

// TWT49U field indices — adjust here if TWSE changes column order
const EX_IDX_CODE: usize = 0;
const EX_IDX_DATE: usize = 2;
const EX_IDX_CLOSE_BEFORE: usize = 3;
const EX_IDX_STOCK_RATE: usize = 4;
const EX_IDX_CASH_DIV: usize = 5;

use super::twse::{self, TwseResponse};

/// 單次請求能打幾個月的 TWSE。
const MAX_UPSTREAM_FETCHES: usize = 6;
/// 單次請求花在上游的時間上限。
const UPSTREAM_TIME_BUDGET: Duration = Duration::from_secs(8);

/// 互動端點對上游（TWSE）的抓取預算。
///
/// **為什麼必須有**：`fetch_all_closing_prices` 是逐月抓，而 `services::twse` 有全域
/// `semaphore(1)`。沒有預算的話，一個三年前買入、持股十檔的 member 按一次 summary
/// 就是 ~360 次序列上游請求（每次 timeout 30 秒）—— 那個請求本身撐不到回應，還會把
/// TWSE 通道從排程 job 手上整段搶走。
///
/// 逾預算的月份直接當成「沒資料」回空：**已抓到的都寫進了 `stock_closing_prices`**，
/// 下次請求會從 DB 命中並接著往前補，幾次之後就完整。所以代價是「剛加入的舊持股，
/// 歷史圖要多按幾次才長齊」，換到的是「任何一次請求都有上限」。
///
/// clone 共用同一份額度與同一個 deadline（summary 對多筆持股平行抓時要算成一份預算）。
#[derive(Clone)]
struct UpstreamBudget {
    deadline: Instant,
    remaining: Arc<AtomicUsize>,
}

impl UpstreamBudget {
    fn new() -> Self {
        Self {
            deadline: Instant::now() + UPSTREAM_TIME_BUDGET,
            remaining: Arc::new(AtomicUsize::new(MAX_UPSTREAM_FETCHES)),
        }
    }

    /// 取一次額度；已用完或已逾時回 false（呼叫端不得再打上游）。
    fn try_take(&self) -> bool {
        if Instant::now() >= self.deadline {
            return false;
        }
        // fetch_update：額度歸零後不再往下減，避免 wrap
        self.remaining
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                (n > 0).then(|| n - 1)
            })
            .is_ok()
    }
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

pub async fn get_by_member(pool: &Pool<Postgres>, member_id: i64) -> Result<Vec<PortfolioEntry>, AppError> {
    portfolio_repo::get_by_member(pool, member_id).await
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    req.validate(crate::utils::date::taipei_today())
        .map_err(crate::errors::RequestError::UnprocessableContent)?;
    portfolio_repo::create(pool, member_id, req).await
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    req.validate(crate::utils::date::taipei_today())
        .map_err(crate::errors::RequestError::UnprocessableContent)?;
    portfolio_repo::update(pool, id, member_id, req).await
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    portfolio_repo::delete(pool, id, member_id).await
}

pub async fn get_history(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    client: &Client,
    id: Uuid,
    member_id: i64,
) -> Result<Vec<HistoryRecord>, AppError> {
    let entry = portfolio_repo::get_by_id_for_member(pool, id, member_id).await?;
    let today = crate::utils::date::taipei_today();
    let budget = UpstreamBudget::new();

    let closes = fetch_all_closing_prices(pool, redis_pool, client, &entry.stock_code, entry.buy_date, today, &budget).await?;
    let ex_events = fetch_ex_events(pool, redis_pool, client, &entry.stock_code, entry.buy_date, today, &budget).await?;

    Ok(build_history(entry.cost_per_share, entry.shares, closes, ex_events))
}

pub async fn get_summary(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    client: &Client,
    member_id: i64,
) -> Result<Vec<PortfolioSummaryEntry>, AppError> {
    let entries = portfolio_repo::get_by_member(pool, member_id).await?;
    let today = crate::utils::date::taipei_today();
    // 一份預算給整個 summary（多筆持股共用），不是每筆一份
    let budget = UpstreamBudget::new();

    let result = try_join_all(entries.into_iter().map(|entry| {
        let pool = pool.clone();
        let redis_pool = redis_pool.clone();
        let client = client.clone();
        let budget = budget.clone();
        async move {
            let (closes, ex_events, stock_name) = tokio::try_join!(
                fetch_all_closing_prices(&pool, &redis_pool, &client, &entry.stock_code, entry.buy_date, today, &budget),
                fetch_ex_events(&pool, &redis_pool, &client, &entry.stock_code, entry.buy_date, today, &budget),
                async { Ok::<_, AppError>(get_stock_name_by_code(&pool, &entry.stock_code).await.unwrap_or(None)) },
            )?;

            let (current_price, current_value, pnl, pnl_pct) =
                match compute_latest(entry.cost_per_share, entry.shares, &closes, ex_events) {
                    Some((cp, cv, p, pp)) => (Some(cp), Some(cv), Some(p), Some(pp)),
                    None => (None, None, None, None),
                };

            Ok::<_, AppError>(PortfolioSummaryEntry {
                base: entry,
                stock_name,
                current_price,
                current_value,
                pnl,
                pnl_pct,
            })
        }
    }))
    .await?;

    Ok(result)
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
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    client: &Client,
    stock_code: &str,
    month: NaiveDate,
    budget: &UpstreamBudget,
) -> Result<Vec<DayClose>, AppError> {
    let cache_key = format!("twse:stock_day:{}:{}", stock_code, month.format("%Y%m"));
    let today = crate::utils::date::taipei_today();
    let is_current = month.year() == today.year() && month.month() == today.month();
    let ttl = if is_current { 3600u64 } else { 604800u64 };

    // 1. Redis
    if let Ok(Some(cached)) = redis_repo::cache_get(redis_pool, &cache_key).await {
        if let Some(data) = redis_deserialize_closes(&cached) {
            return Ok(data);
        }
    }

    // 2. DB — past months only (historical data is complete; current month may be partial)
    if !is_current {
        let first_day = month;
        let last_day = month
            .checked_add_months(Months::new(1))
            .and_then(|d| d.pred_opt())
            .unwrap_or(month);

        let db_rows = get_stock_closing_prices_by_date_range(pool, stock_code, first_day, last_day).await?;
        if !db_rows.is_empty() {
            let closes: Vec<DayClose> = db_rows.iter().map(|r| DayClose { date: r.date, close: r.close_price }).collect();
            if let Some(json) = redis_serialize_closes(&closes) {
                cache_set_logged(redis_pool, &cache_key, &json, ttl).await;
            }
            return Ok(closes);
        }
    }

    // 3. TWSE（受單次請求的預算限制；逾預算當成沒資料，下次請求再補）
    if !budget.try_take() {
        tracing::debug!(
            "portfolio 上游預算已用盡，跳過 {}/{}",
            stock_code,
            month.format("%Y%m")
        );
        return Ok(vec![]);
    }
    let resp: TwseResponse = match twse::fetch_stock_day(client, stock_code, month).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("TWSE STOCK_DAY fetch failed {}/{}: {}", stock_code, month.format("%Y%m"), e);
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
                let close = twse::parse_f64(&row[6])?;
                Some(DayClose { date, close })
            })
            .collect()
    } else {
        vec![]
    };

    // 4. Write DB
    if !closes.is_empty() {
        let prices: Vec<NewStockClosingPrice> = closes
            .iter()
            .map(|d| NewStockClosingPrice { stock_no: stock_code.to_string(), date: d.date, close_price: d.close })
            .collect();
        if let Err(e) = upsert_stock_closing_prices(pool, &prices).await {
            tracing::warn!("upsert_stock_closing_prices failed {}: {}", stock_code, e);
        }
    }

    // 5. Write Redis
    if let Some(json) = redis_serialize_closes(&closes) {
        cache_set_logged(redis_pool, &cache_key, &json, ttl).await;
    }

    Ok(closes)
}

/// 寫快取失敗要留痕 —— 這幾條路徑原本是 `let _ = cache_set(...)`，於是 Redis 半死時
/// 症狀只有「頁面變慢 + 一直打 TWSE」，log 裡沒有任何線索指向快取。
/// 失敗本身不該讓請求失敗（資料已經算出來了），所以吞掉回傳值、只記 WARN。
async fn cache_set_logged(
    redis_pool: &RedisPool<RedisConnectionManager>,
    key: &str,
    json: &str,
    ttl: u64,
) {
    if let Err(e) = redis_repo::cache_set(redis_pool, key, json, ttl).await {
        tracing::warn!("portfolio 快取寫入失敗 key={}: {}", key, e);
    }
}

async fn fetch_all_closing_prices(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    client: &Client,
    stock_code: &str,
    from: NaiveDate,
    to: NaiveDate,
    budget: &UpstreamBudget,
) -> Result<Vec<DayClose>, AppError> {
    // 迴圈起點夾在 MIN_BUY_DATE：寫入端的 `PortfolioRequest::validate` 只擋得住新資料，
    // 這道是給**存量列**的 —— 驗證是後來才補的，在那之前寫進來的 buy_date 沒有下限，
    // 而這個迴圈每個月都要打一次 Redis 加一次 DB。比 1992 更早的月份 TWSE 本來就沒有
    // 資料，夾掉只是省下白跑的查詢，不會少算任何東西。
    let from_month = from.max(crate::structs::portfolio::min_buy_date());

    let mut months = Vec::new();
    let mut current =
        NaiveDate::from_ymd_opt(from_month.year(), from_month.month(), 1).expect("每月必有 1 日");
    let end_month = NaiveDate::from_ymd_opt(to.year(), to.month(), 1).expect("每月必有 1 日");
    while current <= end_month {
        months.push(current);
        let Some(next) = current.checked_add_months(Months::new(1)) else { break };
        current = next;
    }

    // **由新到舊抓**：上游預算有限時，額度要先花在最新的月份 —— summary 的現價與
    // history 的最右端都取自最後一筆收盤價。由舊到新會把額度耗在最舊的月份上，
    // 結果是最該有的現價反而拿不到。最後統一排序，順序對呼叫端不可見。
    let mut all: Vec<DayClose> = Vec::new();
    for month in months.into_iter().rev() {
        let mut month_data =
            fetch_closing_month(pool, redis_pool, client, stock_code, month, budget).await?;
        all.append(&mut month_data);
    }

    all.retain(|d| d.date >= from);
    all.sort_by_key(|d| d.date);
    Ok(all)
}

async fn fetch_ex_events(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    client: &Client,
    stock_code: &str,
    from: NaiveDate,
    to: NaiveDate,
    budget: &UpstreamBudget,
) -> Result<Vec<ExEvent>, AppError> {
    let start_str = from.format("%Y%m%d").to_string();
    let end_str = to.format("%Y%m%d").to_string();
    let cache_key = format!("twse:exright:{}:{}", stock_code, start_str);

    // 1. Redis
    if let Ok(Some(cached)) = redis_repo::cache_get(redis_pool, &cache_key).await {
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
            return Ok(events);
        }
    }

    // 2. DB (ex-rights rows)
    let db_rows = get_ex_rights_by_range(pool, stock_code, from, to).await?;
    if !db_rows.is_empty() {
        let events: Vec<ExEvent> = db_rows
            .iter()
            .map(|r| ExEvent { date: r.ex_date, close_before: r.close_before, cash_div: r.cash_div, stock_rate: r.stock_rate })
            .collect();
        cache_ex_events(redis_pool, &cache_key, &events).await;
        return Ok(events);
    }

    // 2.5. DB (checked table) — confirmed no ex-rights within 30 days
    if let Ok(Some(checked_at)) = find_ex_rights_checked(pool, stock_code, from).await {
        let age_days = (chrono::Utc::now() - checked_at).num_days();
        if age_days < 30 {
            cache_ex_events(redis_pool, &cache_key, &[]).await;
            return Ok(vec![]);
        }
    }

    // 3. TWSE（同一份請求預算）。
    // 逾預算必須在這裡就回，不能往下走 —— 下面 4.5 的 `upsert_ex_rights_checked` 代表
    // 「已向 TWSE 確認過這 30 天沒有除權息」，沒真的問就寫等於騙了自己 30 天。
    if !budget.try_take() {
        tracing::debug!("portfolio 上游預算已用盡，跳過 {stock_code} 的除權息查詢");
        return Ok(vec![]);
    }
    let resp: TwseResponse = match twse::fetch_ex_rights(client, &start_str, &end_str).await {
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
                let close_before = twse::parse_f64(&row[EX_IDX_CLOSE_BEFORE]).unwrap_or(0.0);
                let stock_rate = twse::parse_f64(&row[EX_IDX_STOCK_RATE]).unwrap_or(0.0);
                let cash_div = twse::parse_f64(&row[EX_IDX_CASH_DIV]).unwrap_or(0.0);
                Some(ExEvent { date, close_before, cash_div, stock_rate })
            })
            .collect()
    } else {
        vec![]
    };

    // 4. Write DB
    if !events.is_empty() {
        let rows: Vec<StockExRight> = events
            .iter()
            .map(|e| StockExRight { stock_no: stock_code.to_string(), ex_date: e.date, close_before: e.close_before, cash_div: e.cash_div, stock_rate: e.stock_rate })
            .collect();
        if let Err(e) = upsert_ex_rights(pool, &rows).await {
            tracing::warn!("upsert_ex_rights failed {}: {}", stock_code, e);
        }
    }

    // 4.5. Write checked record (regardless of result, marks TWSE was queried)
    if let Err(e) = upsert_ex_rights_checked(pool, stock_code, from).await {
        tracing::warn!("upsert_ex_rights_checked failed {}: {}", stock_code, e);
    }

    // 5. Write Redis
    cache_ex_events(redis_pool, &cache_key, &events).await;

    Ok(events)
}

async fn cache_ex_events(redis_pool: &RedisPool<RedisConnectionManager>, key: &str, events: &[ExEvent]) {
    let v: Vec<(String, f64, f64, f64)> = events
        .iter()
        .map(|e| (e.date.format("%Y-%m-%d").to_string(), e.close_before, e.cash_div, e.stock_rate))
        .collect();
    if let Ok(json) = serde_json::to_string(&v) {
        cache_set_logged(redis_pool, key, &json, 86400).await;
    }
}

fn compute_latest(
    cost: f64,
    shares: i64,
    closes: &[DayClose],
    mut ex_events: Vec<ExEvent>,
) -> Option<(f64, f64, f64, f64)> {
    let last = closes.last()?;
    ex_events.sort_by_key(|e| e.date);

    let mut adjusted_cost = cost;
    for ev in &ex_events {
        if ev.date > last.date {
            break;
        }
        if ev.close_before > 0.0 {
            let numer = ev.close_before - ev.cash_div;
            let denom = ev.close_before * (1.0 + ev.stock_rate / 1000.0);
            if denom > 0.0 {
                adjusted_cost = adjusted_cost * numer / denom;
            }
        }
    }

    let pnl = (last.close - adjusted_cost) * shares as f64;
    let pnl_pct = if adjusted_cost != 0.0 {
        (last.close - adjusted_cost) / adjusted_cost * 100.0
    } else {
        0.0
    };

    Some((last.close, last.close * shares as f64, pnl, pnl_pct))
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
