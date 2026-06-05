use crate::{
    repositories::stocks::{get_active_buyback_prices_filtered, upsert_stock_closing_prices},
    services::stocks::{get_stock_day_avg, parse_stock_day_avg_response},
    state::AppState,
    structs::{jobs::AppJob, stocks::StartPriceFilter},
};
use async_trait::async_trait;

pub struct FetchHistoricalClosingPricesJob;

#[async_trait]
impl AppJob for FetchHistoricalClosingPricesJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *"
    }

    async fn run(&self, state: AppState) {
        let no_start_price_data =
            match get_active_buyback_prices_filtered(&state, StartPriceFilter::MissingOnly).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("{}", e.to_string());
                    return;
                }
            };

        // Take the oldest entry first; one per minute to avoid TWSE rate limiting
        if let Some(data) = no_start_price_data.into_iter().next() {
            let avg_response = match get_stock_day_avg(state.get_http_client(), &data.stock_no, data.start_date).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!("get_stock_day_avg fail stock_no={} date={}: {}", data.stock_no, data.start_date, e);
                    return;
                }
            };

            let new_stock_closing_prices = parse_stock_day_avg_response(avg_response, &data.stock_no);

            if let Err(e) = upsert_stock_closing_prices(&state, &new_stock_closing_prices).await {
                tracing::error!("upsert_stock_closing_prices fail stock_no={} date={}: {}", data.stock_no, data.start_date, e);
                return;
            }

            tracing::info!("{} {} upsert_stock_closing_prices success", data.stock_no, data.start_date);
        }
    }
}
