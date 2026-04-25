use crate::{
    errors::{AppError, RequestError},
    state::AppStateV2,
    structs::stocks::{StartPriceFilter, StockBuybackInfo, StockBuybackMoreInfo, StockBuybackPeriod, StockRequest},
};
use chrono::NaiveDate;
use sqlx::QueryBuilder;

pub async fn bulk_insert_stock_buyback_periods(
    state: &AppStateV2,
    stocks: &[StockRequest],
) -> Result<u64, AppError> {
    let stock_nos: Vec<&str> = stocks.iter().map(|s| s.stock_no.as_str()).collect();

    let start_dates: Result<Vec<NaiveDate>, AppError> = stocks
        .iter()
        .map(|s| roc_date_to_naive_date(s.start_date.as_str()))
        .collect();
    let start_dates = start_dates?;

    let end_dates: Result<Vec<NaiveDate>, AppError> = stocks
        .iter()
        .map(|s| roc_date_to_naive_date(s.end_date.as_str()))
        .collect();
    let end_dates = end_dates?;

    let result = sqlx::query(
        "INSERT INTO stock_buyback_periods (stock_no, start_date, end_date)
        SELECT * FROM UNNEST($1::text[], $2::date[], $3::date[])
        ON CONFLICT (stock_no, start_date, end_date) DO NOTHING",
    )
    .bind(&stock_nos)
    .bind(&start_dates)
    .bind(&end_dates)
    .execute(state.get_pool())
    .await?;

    Ok(result.rows_affected())
}

pub async fn get_active_buyback_prices(
    state: &AppStateV2,
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

pub async fn get_active_buyback_prices_v4(
    state: &AppStateV2,
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

pub async fn get_stock_buyback_periods(
    state: &AppStateV2,
) -> Result<Vec<StockBuybackPeriod>, AppError> {
    Ok(
        sqlx::query_as("SELECT * FROM stock_buyback_periods ORDER BY start_date ASC")
            .fetch_all(state.get_pool())
            .await?,
    )
}

fn roc_date_to_naive_date(roc_date: &str) -> Result<NaiveDate, AppError> {
    if roc_date.len() != 7 {
        return Err(RequestError::InvalidContent("無效的民國日期格式".into()).into());
    }

    let roc_year = &roc_date[0..3];
    let month = &roc_date[3..5];
    let day = &roc_date[5..7];

    let roc_year: i32 = roc_year
        .parse()
        .map_err(|_| AppError::from(RequestError::InvalidContent(format!("民國年解析失敗: {}", roc_year))))?;
    let month: u32 = month
        .parse()
        .map_err(|_| AppError::from(RequestError::InvalidContent(format!("月份解析失敗: {}", month))))?;
    let day: u32 = day
        .parse()
        .map_err(|_| AppError::from(RequestError::InvalidContent(format!("日期解析失敗: {}", day))))?;

    chrono::NaiveDate::from_ymd_opt(roc_year + 1911, month, day)
        .ok_or_else(|| RequestError::InvalidContent("創建 NaiveDate fail".into()).into())
}
