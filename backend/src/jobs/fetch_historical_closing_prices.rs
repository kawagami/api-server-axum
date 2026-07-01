use crate::{
    repositories::stocks::{get_active_buyback_prices_filtered, upsert_stock_closing_prices},
    services::stocks::{get_stock_day_avg, parse_stock_day_avg_response},
    state::AppState,
    structs::stocks::StartPriceFilter,
};

pub async fn run(state: AppState) {
    let pool = state.get_pool();
    let client = state.get_http_client();

    let no_start_price_data =
        match get_active_buyback_prices_filtered(pool, StartPriceFilter::MissingOnly).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("{}", e.to_string());
                return;
            }
        };

    // Take the oldest entry first; one per minute to avoid TWSE rate limiting
    if let Some(data) = no_start_price_data.into_iter().next() {
        let avg_response = match get_stock_day_avg(client, &data.stock_no, data.start_date).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("get_stock_day_avg fail stock_no={} date={}: {}", data.stock_no, data.start_date, e);
                return;
            }
        };

        let new_stock_closing_prices = parse_stock_day_avg_response(avg_response, &data.stock_no);

        if let Err(e) = upsert_stock_closing_prices(pool, &new_stock_closing_prices).await {
            tracing::error!("upsert_stock_closing_prices fail stock_no={} date={}: {}", data.stock_no, data.start_date, e);
            return;
        }

        tracing::info!("{} {} upsert_stock_closing_prices success", data.stock_no, data.start_date);
    }
}
