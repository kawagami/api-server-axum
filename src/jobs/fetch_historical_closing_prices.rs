use crate::{
    repositories::stocks::{get_active_buyback_prices_v4, upsert_stock_closing_prices},
    services::stocks::{get_stock_day_avg, parse_stock_day_avg_response},
    state::AppStateV2,
    structs::{jobs::AppJob, stocks::StartPriceFilter},
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct FetchHistoricalClosingPricesJob;

#[async_trait]
impl AppJob for FetchHistoricalClosingPricesJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *" // 每分鐘執行一次
    }

    /// 定時打外部 API 取歷史收盤價
    async fn run(&self, state: AppStateV2) {
        // 取 stock_no date
        let mut no_start_price_data =
            match get_active_buyback_prices_v4(&state, StartPriceFilter::MissingOnly).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("{}", e.to_string());
                    return;
                }
            };

        //
        if let Some(data) = no_start_price_data.pop() {
            let new_stock_closing_prices = parse_stock_day_avg_response(
                get_stock_day_avg(
                    state.get_http_client(),
                    &data.stock_no,
                    &data.start_date.format("%Y%m%d").to_string(),
                )
                .await
                .expect(&format!(
                    "get_stock_day_avg fail, stock_no => {}, start_date => {}",
                    &data.stock_no,
                    &data.start_date.format("%Y%m%d").to_string()
                )),
                &data.stock_no,
            );

            // 將歷史價寫進資料庫 stock_closing_prices 只記錄特定股票在特定日的收盤價
            upsert_stock_closing_prices(&state, &new_stock_closing_prices)
                .await
                .expect("msg");

            tracing::info!(
                "{} {} upsert_stock_closing_prices success",
                &data.stock_no,
                &data.start_date.format("%Y%m%d").to_string()
            );
        }
    }
}
