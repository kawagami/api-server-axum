use crate::{
    repositories::stocks,
    services::{
        email::send_notification,
        stocks::{get_buyback_stock_raw_html_string, parse_buyback_stock_raw_html},
    },
    state::AppState,
};
use chrono::{Datelike, Duration, Local, Months, NaiveDate};

pub async fn run(state: AppState) {
    let today = Local::now().naive_local().date();
    let six_months_ago = today - Duration::days(180);
    let three_months_later = today
        .checked_add_months(Months::new(3))
        .expect("overflow adding 3 months");

    let start = date_to_roc_string(six_months_ago);
    let end = date_to_roc_string(three_months_later);

    match get_buyback_stock_raw_html_string(state.get_http_client(), &start, &end).await {
        Ok(html_string) => {
            let records = parse_buyback_stock_raw_html(html_string);
            tracing::info!("parsed {} buyback records ({} ~ {})", records.len(), start, end);
            match stocks::bulk_insert_stock_buyback_periods(state.get_pool(), &records).await {
                Ok(n) => {
                    tracing::info!("bulk_insert_stock_buyback_periods inserted {} rows", n);
                    notify_new_future_buybacks(&state).await;
                }
                Err(e) => tracing::error!("bulk_insert_stock_buyback_periods fail: {}", e),
            }
        }
        Err(e) => tracing::error!("get_buyback_stock_raw_html_string fail: {}", e),
    }
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
