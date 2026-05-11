use crate::{
    repositories::stocks,
    services::stocks::{get_buyback_stock_raw_html_string, parse_buyback_stock_raw_html},
    state::AppState,
    structs::jobs::AppJob,
};
use async_trait::async_trait;
use chrono::{Datelike, Local, Months, NaiveDate};

#[derive(Clone)]
pub struct FetchBuybackPeriodsJob;

#[async_trait]
impl AppJob for FetchBuybackPeriodsJob {
    fn cron_expression(&self) -> &str {
        "0 0 20 * * *"
    }

    async fn run(&self, state: AppState) {
        let today = Local::now().naive_local().date();
        let three_months_later = today
            .checked_add_months(Months::new(3))
            .expect("overflow adding 3 months");

        let start = date_to_roc_string(today);
        let end = date_to_roc_string(three_months_later);

        match get_buyback_stock_raw_html_string(state.get_http_client(), &start, &end).await {
            Ok(html_string) => {
                let records = parse_buyback_stock_raw_html(html_string);
                tracing::info!("parsed {} buyback records ({} ~ {})", records.len(), start, end);
                match stocks::bulk_insert_stock_buyback_periods(&state, &records).await {
                    Ok(n) => tracing::info!("bulk_insert_stock_buyback_periods inserted {} rows", n),
                    Err(e) => tracing::error!("bulk_insert_stock_buyback_periods fail: {}", e),
                }
            }
            Err(e) => tracing::error!("get_buyback_stock_raw_html_string fail: {}", e),
        }
    }
}

fn date_to_roc_string(date: NaiveDate) -> String {
    format!("{:03}{}", date.year() - 1911, date.format("%m%d"))
}
