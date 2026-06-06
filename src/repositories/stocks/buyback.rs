use crate::{
    errors::AppError,
    state::AppState,
    structs::stocks::{BuybackRecord, StartPriceFilter, StockBuybackInfo, StockBuybackMoreInfo, StockBuybackPeriod},
};
use sqlx::QueryBuilder;

pub async fn bulk_insert_stock_buyback_periods(
    state: &AppState,
    records: &[BuybackRecord],
) -> Result<u64, AppError> {
    if records.is_empty() {
        return Ok(0);
    }

    let stock_nos: Vec<&str> = records.iter().map(|r| r.stock_no.as_str()).collect();
    let start_dates: Vec<_> = records.iter().map(|r| r.start_date).collect();
    let end_dates: Vec<_> = records.iter().map(|r| r.end_date).collect();

    let result = sqlx::query(
        "INSERT INTO stock_buyback_periods (stock_no, start_date, end_date)
        SELECT * FROM UNNEST($1::text[], $2::date[], $3::date[])
        ON CONFLICT (stock_no, start_date) DO UPDATE SET end_date = EXCLUDED.end_date",
    )
    .bind(&stock_nos)
    .bind(&start_dates)
    .bind(&end_dates)
    .execute(state.get_pool())
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_active_buyback_prices(
    state: &AppState,
) -> Result<Vec<StockBuybackMoreInfo>, AppError> {
    Ok(sqlx::query_as(
        "SELECT
            p.stock_no,
            latest_prices.stock_name,
            p.start_date,
            p.end_date,
            start_date_price.close_price AS price_on_start_date,
            latest_prices.close_price AS latest_price,
            ROUND((latest_prices.close_price - start_date_price.close_price)::numeric, 2) AS diff,
            ROUND(
                CASE
                    WHEN start_date_price.close_price IS NOT NULL AND start_date_price.close_price <> 0
                    THEN ((latest_prices.close_price - start_date_price.close_price) / start_date_price.close_price)::numeric * 100
                    ELSE NULL
                END,
                2
            ) AS diff_percent
        FROM stock_buyback_periods p
        LEFT JOIN (
            SELECT DISTINCT ON (stock_code) stock_code, stock_name, trade_date, close_price
            FROM stock_day_all
            ORDER BY stock_code, trade_date DESC
        ) latest_prices ON latest_prices.stock_code = p.stock_no
        LEFT JOIN LATERAL (
            SELECT close_price FROM stock_closing_prices
            WHERE stock_no = p.stock_no
                AND date BETWEEN p.start_date AND p.start_date + INTERVAL '3 days'
            ORDER BY date ASC LIMIT 1
        ) AS start_date_price ON TRUE
        WHERE p.end_date > CURRENT_DATE
            AND start_date_price.close_price IS NOT NULL
        ORDER BY p.start_date ASC",
    )
    .fetch_all(state.get_pool())
    .await?)
}

pub async fn get_active_buyback_prices_filtered(
    state: &AppState,
    filter: StartPriceFilter,
) -> Result<Vec<StockBuybackInfo>, AppError> {
    let mut qb = QueryBuilder::new(
        "SELECT
            p.stock_no,
            p.start_date,
            p.end_date,
            start_date_price.close_price AS price_on_start_date,
            latest_prices.close_price AS latest_price
        FROM stock_buyback_periods p
        LEFT JOIN (
            SELECT DISTINCT ON (stock_code) stock_code, trade_date, close_price
            FROM stock_day_all
            ORDER BY stock_code, trade_date DESC
        ) latest_prices ON latest_prices.stock_code = p.stock_no
        LEFT JOIN LATERAL (
            SELECT close_price FROM stock_closing_prices
            WHERE stock_no = p.stock_no
                AND date BETWEEN p.start_date AND p.start_date + INTERVAL '3 days'
            ORDER BY date ASC LIMIT 1
        ) AS start_date_price ON TRUE
        WHERE p.end_date > CURRENT_DATE",
    );

    match filter {
        StartPriceFilter::All => {}
        StartPriceFilter::MissingOnly => {
            qb.push(" AND start_date_price.close_price IS NULL AND p.start_date < CURRENT_DATE");
        }
        StartPriceFilter::ExistsOnly => {
            qb.push(" AND start_date_price.close_price IS NOT NULL");
        }
    }

    qb.push(" ORDER BY p.start_date ASC");

    Ok(qb.build_query_as().fetch_all(state.get_pool()).await?)
}

pub async fn get_new_future_buybacks(
    state: &AppState,
) -> Result<Vec<StockBuybackPeriod>, AppError> {
    Ok(sqlx::query_as(
        "SELECT stock_no, start_date, end_date
         FROM stock_buyback_periods
         WHERE start_date > CURRENT_DATE
           AND created_at::date = CURRENT_DATE
         ORDER BY start_date ASC",
    )
    .fetch_all(state.get_pool())
    .await?)
}

pub async fn get_stock_buyback_periods(
    state: &AppState,
) -> Result<Vec<StockBuybackPeriod>, AppError> {
    Ok(
        sqlx::query_as("SELECT * FROM stock_buyback_periods ORDER BY start_date ASC")
            .fetch_all(state.get_pool())
            .await?,
    )
}
