use crate::{
    errors::AppError,
    repositories::{
        portfolio as portfolio_repo,
        redis as redis_repo,
        stocks::{find_ex_rights_checked, get_ex_rights_by_range, get_stock_closing_prices_by_date_range, get_stock_names_by_codes, upsert_ex_rights, upsert_ex_rights_checked, upsert_stock_closing_prices},
    },
    structs::{
        portfolio::{HistoryRecord, PortfolioSummaryEntry},
        stocks::{NewStockClosingPrice, StockExRight},
    },
    utils::date::{parse_roc_cjk_date, parse_roc_date},
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
use super::math::{DayClose, ExEvent, build_history, compute_latest};

/// 除權息一次向 TWSE 查多長。TWT49U 沒有個股篩選、回的是全市場：實測一年約 250 KB / 5 秒，
/// 四年多就要 25 秒、逼近 client 的 30 秒 timeout —— 長區間一定要切段。
const EX_RIGHTS_CHUNK_DAYS: u64 = 365;
/// 已涵蓋到今天的紀錄多久後重查今天：當天的除權息可能是在上次查詢之後才公告。
const EX_RIGHTS_RECHECK_HOURS: i64 = 6;
/// Redis 只擋同一小時內的重複 DB 查詢；真正的新鮮度由 `EX_RIGHTS_RECHECK_HOURS` 決定。
const EX_RIGHTS_CACHE_TTL_SECS: u64 = 3600;

use crate::services::twse::{self, TwseResponse};

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
    // key 帶 `to`：隔天自然 miss，才會回頭去 DB 看涵蓋範圍、往後補新的除權息
    let cache_key = format!(
        "twse:exright:v2:{}:{}:{}",
        stock_code,
        from.format("%Y%m%d"),
        to.format("%Y%m%d")
    );

    // 1. Redis（只存「已涵蓋到 to」的完整結果）
    if let Ok(Some(cached)) = redis_repo::cache_get(redis_pool, &cache_key).await {
        if let Ok(rows) = serde_json::from_str::<Vec<(String, f64, f64)>>(&cached) {
            let events: Vec<ExEvent> = rows
                .into_iter()
                .filter_map(|(d, close_before, ref_price)| {
                    NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                        .ok()
                        .map(|date| ExEvent { date, close_before, ref_price })
                })
                .collect();
            return Ok(events);
        }
    }

    // 2. DB 已向 TWSE 確認到哪一天。從那天（含）接著往後查：那天可能是在當日公告前查的。
    //    舊設計是「DB 有任何一筆就直接用」，於是買進後第一次配息之後的每一次配息都補不進來。
    let mut next = match find_ex_rights_checked(pool, stock_code, from).await? {
        Some((until, checked_at))
            if until >= to && (chrono::Utc::now() - checked_at).num_hours() < EX_RIGHTS_RECHECK_HOURS =>
        {
            None
        }
        Some((until, _)) => Some(until.min(to)),
        None => Some(from),
    };
    let mut complete = next.is_none();

    // 3. TWSE，由舊到新分段（與收盤價相反：還原因子要從買進日一路累乘，缺舊的整段都錯，
    //    而進度記在 covered_until，下次請求從斷點接著查，不會重打已確認的段）
    while let Some(start) = next {
        let end = start
            .checked_add_days(Days::new(EX_RIGHTS_CHUNK_DAYS - 1))
            .map_or(to, |d| d.min(to));

        // 逾預算必須在寫 checked 之前停 —— 那筆紀錄代表「這段已確認過」，沒真的問就寫等於騙自己
        if !budget.try_take() {
            tracing::debug!("portfolio 上游預算已用盡，{stock_code} 的除權息停在 {start}，下次接著查");
            break;
        }
        let Some(events) = fetch_ex_chunk(client, stock_code, start, end).await else { break };

        if !events.is_empty() {
            let rows: Vec<StockExRight> = events
                .iter()
                .map(|e| StockExRight {
                    stock_no: stock_code.to_string(),
                    ex_date: e.date,
                    close_before: e.close_before,
                    ref_price: e.ref_price,
                })
                .collect();
            if let Err(e) = upsert_ex_rights(pool, &rows).await {
                tracing::warn!("upsert_ex_rights failed {}: {}", stock_code, e);
                break;
            }
        }
        if let Err(e) = upsert_ex_rights_checked(pool, stock_code, from, end).await {
            tracing::warn!("upsert_ex_rights_checked failed {}: {}", stock_code, e);
            break;
        }

        if end >= to {
            complete = true;
            break;
        }
        next = end.succ_opt();
    }

    // 4. 結果一律從 DB 讀：中途停下時就是「已確認那段」的部分結果，下次請求接著補
    let events: Vec<ExEvent> = get_ex_rights_by_range(pool, stock_code, from, to)
        .await?
        .into_iter()
        .map(|r| ExEvent { date: r.ex_date, close_before: r.close_before, ref_price: r.ref_price })
        .collect();

    // 5. Redis：部分結果不快取，否則會蓋住下次請求的補抓
    if complete {
        cache_ex_events(redis_pool, &cache_key, &events).await;
    }

    Ok(events)
}

/// 查一段期間的除權息。回 `None` = 這段**不能**當成已確認（上游失敗或回應格式變了），
/// 呼叫端不得寫 checked。
async fn fetch_ex_chunk(
    client: &Client,
    stock_code: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Option<Vec<ExEvent>> {
    let start_str = start.format("%Y%m%d").to_string();
    let end_str = end.format("%Y%m%d").to_string();
    let resp = match twse::fetch_ex_rights(client, &start_str, &end_str).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("TWSE TWT49U fetch failed {}/{}-{}: {}", stock_code, start_str, end_str, e);
            return None;
        }
    };
    let parsed = parse_ex_rights(&resp, stock_code);
    if parsed.is_none() {
        // 這條以前是靜默的：解析不到就當「沒有除權息」並標記已確認，整整壞了好幾個月沒人發現
        tracing::warn!(
            "TWT49U 回應無法解析，不標記已確認 {}/{}-{} stat={} fields={:?}",
            stock_code,
            start_str,
            end_str,
            resp.stat,
            resp.fields
        );
    }
    parsed
}

/// 從 TWT49U（全市場）回應挑出單一股票的除權息。純函式。
///
/// 回 `None` 表示回應不可信：stat 非 OK（查無資料除外）、找不到必要欄名、或代號對上了
/// 日期卻解析不出來 —— 這些都是上游改版的徵兆，**不能**當成「確認過沒有」。
fn parse_ex_rights(resp: &TwseResponse, stock_code: &str) -> Option<Vec<ExEvent>> {
    if resp.is_no_data() {
        return Some(vec![]);
    }
    if resp.stat != "OK" {
        return None;
    }
    let [i_date, i_code, i_close, i_ref] =
        resp.field_indices(["資料日期", "股票代號", "除權息前收盤價", "減除股利參考價"])?;

    let mut events = Vec::new();
    for row in resp.data.as_deref().unwrap_or_default() {
        if row.get(i_code).map(|c| c.trim()) != Some(stock_code) {
            continue;
        }
        let date = parse_roc_cjk_date(row.get(i_date)?)?;
        // 價格是 "--"（例如只有現金增資、沒有配股配息）= 沒有要還原的，跳過這筆而不是整段作廢，
        // 否則那檔股票每次請求都會重打同一段上游
        let (Some(close_before), Some(ref_price)) = (
            row.get(i_close).and_then(|s| twse::parse_f64(s)),
            row.get(i_ref).and_then(|s| twse::parse_f64(s)),
        ) else {
            continue;
        };
        events.push(ExEvent { date, close_before, ref_price });
    }
    Some(events)
}

async fn cache_ex_events(redis_pool: &RedisPool<RedisConnectionManager>, key: &str, events: &[ExEvent]) {
    let v: Vec<(String, f64, f64)> = events
        .iter()
        .map(|e| (e.date.format("%Y-%m-%d").to_string(), e.close_before, e.ref_price))
        .collect();
    if let Ok(json) = serde_json::to_string(&v) {
        cache_set_logged(redis_pool, key, &json, EX_RIGHTS_CACHE_TTL_SECS).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELDS: [&str; 11] = [
        "資料日期", "股票代號", "股票名稱", "除權息前收盤價", "除權息參考價", "權值+息值",
        "權/息", "漲停價格", "跌停價格", "開盤競價基準", "減除股利參考價",
    ];

    fn resp(stat: &str, fields: &[&str], rows: &[&[&str]]) -> TwseResponse {
        TwseResponse {
            stat: stat.to_string(),
            fields: fields.iter().map(|s| s.to_string()).collect(),
            data: Some(rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect()),
        }
    }

    // 2026-09 實際打 TWT49U 取回的列（截到「減除股利參考價」）
    const TSMC: &[&str] = &[
        "114年09月16日", "2330", "台積電", "1,255.00", "1,249.99", "5.000017",
        "息", "1,370.00", "1,125.00", "1,250.00", "1,249.99",
    ];
    const OTHER: &[&str] = &[
        "115年09月29日", "2109", "華豐", "15.00", "14.50", "0.500000",
        "息", "16.50", "13.50", "14.50", "14.50",
    ];

    #[test]
    fn picks_only_the_requested_stock() {
        let ev = parse_ex_rights(&resp("OK", &FIELDS, &[OTHER, TSMC]), "2330").expect("可解析");

        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].date, NaiveDate::from_ymd_opt(2025, 9, 16).expect("日期"));
        assert_eq!(ev[0].close_before, 1255.0);
        assert_eq!(ev[0].ref_price, 1249.99);
    }

    #[test]
    fn no_data_stat_is_a_confirmed_empty_result() {
        let r = TwseResponse { stat: "很抱歉，沒有符合條件的資料!".into(), fields: vec![], data: None };
        assert_eq!(parse_ex_rights(&r, "2330").map(|v| v.len()), Some(0));
    }

    #[test]
    fn unknown_stat_is_not_trusted() {
        // 被擋、維護中之類的回應：不能當成「確認過沒有」
        assert!(parse_ex_rights(&resp("查詢日期大於今日", &FIELDS, &[]), "2330").is_none());
    }

    #[test]
    fn missing_field_name_is_not_trusted() {
        // 上游改版拿掉或改名欄位 —— 舊版依固定索引讀，正是這樣靜默讀錯欄好幾個月
        let fields: Vec<&str> = FIELDS.iter().copied().filter(|f| *f != "減除股利參考價").collect();
        assert!(parse_ex_rights(&resp("OK", &fields, &[TSMC]), "2330").is_none());
    }

    #[test]
    fn unparseable_date_on_matching_row_is_not_trusted() {
        let mut row = TSMC.to_vec();
        row[0] = "114/09/16";
        assert!(parse_ex_rights(&resp("OK", &FIELDS, &[&row]), "2330").is_none());
    }

    #[test]
    fn row_without_reference_price_is_skipped() {
        let mut row = TSMC.to_vec();
        row[10] = "--";
        assert_eq!(parse_ex_rights(&resp("OK", &FIELDS, &[&row]), "2330").map(|v| v.len()), Some(0));
    }
}
