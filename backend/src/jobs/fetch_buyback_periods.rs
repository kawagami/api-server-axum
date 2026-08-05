use crate::{
    errors::AppError,
    repositories::stocks,
    services::{
        email::send_notification,
        stocks::{get_buyback_stock_raw_html_string, parse_buyback_stock_raw_html},
    },
    state::AppState,
};
use chrono::{Datelike, Duration, Months, NaiveDate};

pub async fn run(state: AppState) {
    let today = crate::utils::date::taipei_today();
    let six_months_ago = today - Duration::days(180);
    // NaiveDate 加 3 個月實務上不會溢位，但同函式其他錯誤都走 log，這裡不該是唯一的 panic 點
    let three_months_later = today.checked_add_months(Months::new(3)).unwrap_or(today);

    let start = date_to_roc_string(six_months_ago);
    let end = date_to_roc_string(three_months_later);

    // 補上重試：這是每日一次的外部抓取，性質與 fetch_stock_day_all / fetch_gov_tenders 相同。
    // TWSE 抽風一次就整天沒有新的庫藏股資料，而 10 分鐘後 SyncBuybackToPending 會拿舊資料跑。
    super::run_with_retries(
        "fetch_buyback_periods",
        3,
        std::time::Duration::from_secs(3600),
        || fetch_and_store(&state, &start, &end),
    )
    .await;

    // 無條件呼叫：它自己會在沒有新資料時提早 return，所以抓取失敗時是 no-op，
    // 而上一輪插入成功但通知失敗的情況這裡還能補寄。
    notify_new_future_buybacks(&state).await;
}

async fn fetch_and_store(state: &AppState, start: &str, end: &str) -> Result<(), AppError> {
    let html_string =
        get_buyback_stock_raw_html_string(state.get_http_client(), start, end).await?;
    let records = parse_buyback_stock_raw_html(html_string);
    tracing::info!("parsed {} buyback records ({} ~ {})", records.len(), start, end);
    let n = stocks::bulk_insert_stock_buyback_periods(state.get_pool(), &records).await?;
    tracing::info!("bulk_insert_stock_buyback_periods inserted {} rows", n);
    Ok(())
}

async fn notify_new_future_buybacks(state: &AppState) {
    let new_records = match stocks::get_new_future_buybacks(state.get_pool()).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("get_new_future_buybacks fail: {}", e);
            return;
        }
    };

    if new_records.is_empty() {
        return;
    }

    let lines: Vec<String> = new_records
        .iter()
        .map(|r| format!("{}: {} ~ {}", r.stock_no, r.start_date, r.end_date))
        .collect();
    let body = format!("新增 {} 筆未來庫藏股：\n\n{}", new_records.len(), lines.join("\n"));

    let settings = state.get_settings();
    send_notification(&settings, "新庫藏股通知", body).await;
}

fn date_to_roc_string(date: NaiveDate) -> String {
    format!("{:03}{}", date.year() - 1911, date.format("%m%d"))
}
