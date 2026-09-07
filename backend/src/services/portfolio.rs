use crate::{
    errors::AppError,
    repositories::{
        portfolio as portfolio_repo,
        redis as redis_repo,
        stocks::{find_ex_rights_checked, get_ex_rights_by_range, get_stock_closing_prices_by_date_range, get_stock_names_by_codes, upsert_ex_rights, upsert_ex_rights_checked, upsert_stock_closing_prices},
    },
    structs::{
        portfolio::{
            HistoryRecord, PeriodChange, PeriodChanges, PortfolioEntry, PortfolioRequest,
            PortfolioSummaryEntry,
        },
        stocks::{NewStockClosingPrice, StockExRight},
    },
    utils::date::parse_roc_date,
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use chrono::{Datelike, Days, Months, NaiveDate};
use futures::stream::{self, StreamExt, TryStreamExt};
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

/// `get_summary` 同時處理幾筆持股。
///
/// **不能無上限**：每筆持股要跑兩個查詢（收盤價、除權息），持股 20 檔的無界 fan-out
/// 就是瞬間 40 個查詢搶那幾條 PG 連線，`acquire_timeout(3s)` 一到整個請求 5xx ——
/// 而且這只是一個 member 按一次 summary。上游那面本來就有 `UpstreamBudget` 擋著，
/// 這裡擋的是 DB 那面。
///
/// 排序不受影響：`buffered` 是「併發執行、依序產出」，回傳順序仍是持股清單的順序。
const SUMMARY_CONCURRENCY: usize = 4;

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

/// `compute_latest` 的結果。欄位一多就不該再用 tuple —— summary 現在要的是
/// 「相對成本的損益」加「相對三個期間起點的增減」兩組數字。
struct LatestSnapshot {
    current_price: f64,
    current_value: f64,
    pnl: f64,
    pnl_pct: f64,
    changes: PeriodChanges,
}

/// 基準日最多可以比目標日早幾天。
///
/// **上下限都有理由**：TWSE 春節連假可以連休 9 天，設太緊會讓農曆年前後的「近一週」
/// 整段變成 `None`；但也不能不設 —— `UpstreamBudget` 逾預算的月份是**整段沒有資料**，
/// 那時「最接近且不晚於目標日的一筆」可能是三個月前，拿它算出來的東西不叫「近一週」，
/// 而使用者從畫面上看不出差別。寧可顯示「-」也不要顯示一個名不副實的數字。
const MAX_BASE_LOOKBACK_DAYS: i64 = 10;

/// 除權息還原因子：把「除權息前」的價格換算成「除權息後」的可比價格。
/// 成本調整與前收盤價調整用的是同一個因子，所以抽出來共用。
fn ex_adjust_factor(ev: &ExEvent) -> Option<f64> {
    if ev.close_before <= 0.0 {
        return None;
    }
    let numer = ev.close_before - ev.cash_div;
    let denom = ev.close_before * (1.0 + ev.stock_rate / 1000.0);
    (denom > 0.0).then_some(numer / denom)
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

    // 股名一次查完（原本每筆持股各一發）。去重：同一檔可以有多筆持股。
    // 查不到股名不是錯誤（新上市 / 還沒抓到行情），失敗一律當成空 map 往下走。
    let mut codes: Vec<String> = entries.iter().map(|e| e.stock_code.clone()).collect();
    codes.sort();
    codes.dedup();
    let names = get_stock_names_by_codes(pool, &codes)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("批次取股名失敗，summary 照回但無股名: {:?}", e);
            std::collections::HashMap::new()
        });

    let result: Vec<PortfolioSummaryEntry> = stream::iter(entries.into_iter().map(|entry| {
        let pool = pool.clone();
        let redis_pool = redis_pool.clone();
        let client = client.clone();
        let budget = budget.clone();
        let stock_name = names.get(&entry.stock_code).cloned();
        async move {
            let (closes, ex_events) = tokio::try_join!(
                fetch_all_closing_prices(&pool, &redis_pool, &client, &entry.stock_code, entry.buy_date, today, &budget),
                fetch_ex_events(&pool, &redis_pool, &client, &entry.stock_code, entry.buy_date, today, &budget),
            )?;

            let latest = compute_latest(entry.cost_per_share, entry.shares, &closes, ex_events);

            Ok::<_, AppError>(PortfolioSummaryEntry {
                base: entry,
                stock_name,
                current_price: latest.as_ref().map(|l| l.current_price),
                current_value: latest.as_ref().map(|l| l.current_value),
                pnl: latest.as_ref().map(|l| l.pnl),
                pnl_pct: latest.as_ref().map(|l| l.pnl_pct),
                changes: latest.map(|l| l.changes).unwrap_or_default(),
            })
        }
    }))
    .buffered(SUMMARY_CONCURRENCY)
    .try_collect()
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
) -> Option<LatestSnapshot> {
    let last = closes.last()?;
    ex_events.sort_by_key(|e| e.date);

    let mut adjusted_cost = cost;
    for ev in &ex_events {
        if ev.date > last.date {
            break;
        }
        if let Some(f) = ex_adjust_factor(ev) {
            adjusted_cost *= f;
        }
    }

    let pnl = (last.close - adjusted_cost) * shares as f64;
    let pnl_pct = if adjusted_cost != 0.0 {
        (last.close - adjusted_cost) / adjusted_cost * 100.0
    } else {
        0.0
    };

    // 三個期間的目標日都用日曆算（使用者說的「近一週」是七天前，不是七個交易日前），
    // 再由 `period_change` 往前找最近的交易日。
    let changes = PeriodChanges {
        day: last
            .date
            .checked_sub_days(Days::new(1))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
        week: last
            .date
            .checked_sub_days(Days::new(7))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
        month: last
            .date
            .checked_sub_months(Months::new(1))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
    };

    Some(LatestSnapshot {
        current_price: last.close,
        current_value: last.close * shares as f64,
        pnl,
        pnl_pct,
        changes,
    })
}

/// 以 `target` 當目標日算一個期間的增減。`closes` 需已按日期遞增排序、`ex_events` 已排序。
///
/// 基準日 = **最後一個不晚於 `target` 的交易日**（市場休市就自然往前落），
/// 太舊則放棄（見 `MAX_BASE_LOOKBACK_DAYS`）。
fn period_change(
    shares: i64,
    closes: &[DayClose],
    ex_events: &[ExEvent],
    last: &DayClose,
    target: NaiveDate,
) -> Option<PeriodChange> {
    let idx = closes.partition_point(|d| d.date <= target).checked_sub(1)?;
    let base = &closes[idx];
    // 同一天沒有增減可言（`target` 落在最後一個交易日當天或之後）
    if base.date >= last.date {
        return None;
    }
    if (target - base.date).num_days() > MAX_BASE_LOOKBACK_DAYS {
        return None;
    }

    // 兩日之間的除權息要還原到基準價上，否則除息日會被當成一次大跌
    let mut base_close = base.close;
    for ev in ex_events {
        if ev.date > base.date && ev.date <= last.date {
            if let Some(f) = ex_adjust_factor(ev) {
                base_close *= f;
            }
        }
    }

    let change = last.close - base_close;
    let change_pct = if base_close != 0.0 {
        change / base_close * 100.0
    } else {
        0.0
    };

    Some(PeriodChange {
        base_date: base.date,
        base_close,
        change,
        change_pct,
        value_change: change * shares as f64,
    })
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
            if let Some(f) = ex_adjust_factor(&ex_events[applied]) {
                adjusted_cost *= f;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().expect("測試日期")
    }

    fn closes(rows: &[(&str, f64)]) -> Vec<DayClose> {
        rows.iter().map(|(dt, c)| DayClose { date: d(dt), close: *c }).collect()
    }

    /// 從 `from` 起連續 n 個交易日（跳週末），收盤價逐日 +1
    fn daily_series(from: &str, n: i64, start_close: f64) -> Vec<DayClose> {
        let mut out = Vec::new();
        let mut date = d(from);
        let mut close = start_close;
        while (out.len() as i64) < n {
            if date.weekday().num_days_from_monday() < 5 {
                out.push(DayClose { date, close });
                close += 1.0;
            }
            date = date.succ_opt().expect("測試日期");
        }
        out
    }

    #[test]
    fn day_change_is_relative_to_previous_trading_day() {
        // 週五 → 下週一：目標日（週日）沒開盤，基準自然落回週五
        let c = closes(&[("2026-09-04", 100.0), ("2026-09-07", 110.0)]);
        let day = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes.day.expect("有前一交易日");

        assert_eq!(day.base_date, d("2026-09-04"));
        assert_eq!(day.base_close, 100.0);
        assert_eq!(day.change, 10.0);
        assert_eq!(day.change_pct, 10.0);
        assert_eq!(day.value_change, 10_000.0);
    }

    #[test]
    fn single_day_has_no_change_in_any_period() {
        let c = closes(&[("2026-09-07", 110.0)]);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        assert!(ch.day.is_none());
        assert!(ch.week.is_none());
        assert!(ch.month.is_none());
    }

    #[test]
    fn ex_dividend_day_is_not_reported_as_a_crash() {
        // 前一日收 100、配息 5 元，除息日開平收 95：帳面是 -5，實際沒漲沒跌。
        // 少了基準價的還原，這天會顯示 -5%（增減數字最容易騙人的地方）。
        let c = closes(&[("2026-09-04", 100.0), ("2026-09-07", 95.0)]);
        let ev = vec![ExEvent { date: d("2026-09-07"), close_before: 100.0, cash_div: 5.0, stock_rate: 0.0 }];
        let day = compute_latest(80.0, 1000, &c, ev).expect("有收盤價").changes.day.expect("有前一交易日");

        assert_eq!(day.base_close, 95.0);
        assert_eq!(day.change, 0.0);
        assert_eq!(day.value_change, 0.0);
    }

    #[test]
    fn week_and_month_pick_the_nearest_trading_day_at_or_before_target() {
        // 2026-08-03(一) 起 30 個交易日，收盤 100,101,…；最後一天是 2026-09-11(五) 收 129
        let c = daily_series("2026-08-03", 30, 100.0);
        let last = c.last().expect("有資料");
        assert_eq!(last.date, d("2026-09-11"));

        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        // 一週前 = 09-04(五)，當天有開盤
        let w = ch.week.expect("有一週前");
        assert_eq!(w.base_date, d("2026-09-04"));
        assert_eq!(w.change, 5.0);
        assert_eq!(w.value_change, 5_000.0);

        // 一個月前 = 08-11(二)，當天有開盤
        let m = ch.month.expect("有一個月前");
        assert_eq!(m.base_date, d("2026-08-11"));
        assert_eq!(m.change, 23.0);
    }

    #[test]
    fn period_is_none_when_position_is_too_new() {
        // 只有三個交易日：有今日、沒有一週前與一個月前
        let c = daily_series("2026-09-09", 3, 100.0);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        assert!(ch.day.is_some());
        assert!(ch.week.is_none());
        assert!(ch.month.is_none());
    }

    #[test]
    fn stale_baseline_is_rejected_rather_than_mislabelled() {
        // 行情有大洞（UpstreamBudget 逾預算的月份會整段沒資料）：目標日往前 10 天內
        // 找不到交易日就回 None —— 顯示「-」勝過拿三個月前的價格謊稱是「近一週」。
        let c = closes(&[("2026-06-01", 100.0), ("2026-09-07", 130.0)]);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價");

        assert!(ch.changes.day.is_none());
        assert!(ch.changes.week.is_none());
        assert!(ch.changes.month.is_none());
    }
}
