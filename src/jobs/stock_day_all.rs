use crate::{
    repositories::stocks,
    services::stocks::{
        get_buyback_stock_raw_html_string, parse_buyback_stock_raw_html, stock_day_all_service,
    },
    state::AppStateV2,
    structs::jobs::AppJob,
};
use async_trait::async_trait;
use chrono::{Local, Months};

#[derive(Clone)]
pub struct StockDayAllJob;

#[async_trait]
impl AppJob for StockDayAllJob {
    fn cron_expression(&self) -> &str {
        "0 0 8,20 * * *" // UTC+8 的 16:00 & 04:00 執行
    }

    async fn run(&self, state: AppStateV2) {
        // 每天抓一次 stock day all 的 API 資料進資料庫
        match stock_day_all_service(&state).await {
            Ok(_) => tracing::info!("job 抓 stock day all 的 API 資料進資料庫成功"),
            Err(e) => tracing::error!("job stock_day_all_service fail: {}", e),
        }

        // 取當前日期字串 & 90 天後的日期字串 抓未來的庫藏股計畫
        let now = get_roc_now();
        let three_month_later = get_roc_three_month_later();
        match get_buyback_stock_raw_html_string(state.get_http_client(), &now, &three_month_later)
            .await
        {
            Ok(html_string) => {
                let records = parse_buyback_stock_raw_html(html_string);
                let _ = stocks::bulk_insert_stock_buyback_periods(&state, &records).await;
                tracing::info!(
                    "job get_buyback_stock_raw_html_string 成功 執行時間 {} ~ {}",
                    now,
                    three_month_later
                );
            }
            Err(e) => tracing::error!("job get_buyback_stock_raw_html_string fail: {}", e),
        }
    }
}

/// 取得當下的民國日期格式格式字串：1140630
fn get_roc_now() -> String {
    let now = Local::now().naive_local().date();
    let year = format_year_to_roc(now.format("%Y").to_string());
    let month = now.format("%m").to_string();
    let day = now.format("%d").to_string();
    format!("{}{}{}", year, month, day)
}

/// 取得三個月後的民國日期格式字串：1140930
fn get_roc_three_month_later() -> String {
    let now = Local::now()
        .naive_local()
        .date()
        .checked_add_months(Months::new(3))
        .unwrap();
    let year = format_year_to_roc(now.format("%Y").to_string());
    let month = now.format("%m").to_string();
    let day = now.format("%d").to_string();
    format!("{}{}{}", year, month, day)
}

/// 將西元年轉換成民國年
fn format_year_to_roc(year: String) -> String {
    (year.parse::<i32>().unwrap() - 1911).to_string()
}
