use crate::{errors::AppError, state::AppState, structs::stocks::StockDayAll};
use chrono::NaiveDate;
use sqlx::QueryBuilder;

pub async fn get_stock_day_all(
    state: &AppState,
    stock_code: Option<String>,
    trade_date: Option<NaiveDate>,
    limit: i64,
    offset: i64,
) -> Result<Vec<StockDayAll>, AppError> {
    let mut builder = QueryBuilder::new("SELECT * FROM stock_day_all");

    let mut has_where = false;

    if stock_code.is_some() || trade_date.is_some() {
        builder.push(" WHERE ");
    }

    if let Some(code) = stock_code {
        builder.push("stock_code = ").push_bind(code);
        has_where = true;
    }

    if let Some(date) = trade_date {
        if has_where {
            builder.push(" AND ");
        }
        builder.push("trade_date = ").push_bind(date);
    }

    builder.push(" ORDER BY trade_date DESC, stock_code ASC");
    builder.push(" LIMIT ").push_bind(limit);
    builder.push(" OFFSET ").push_bind(offset);

    Ok(builder.build_query_as::<StockDayAll>().fetch_all(state.get_pool()).await?)
}

pub async fn insert_stock_day_all_batch(
    state: &AppState,
    trade_dates: &[NaiveDate],
    stock_codes: &[String],
    stock_names: &[String],
    trade_volumes: &[i64],
    trade_amounts: &[i64],
    open_prices: &[f64],
    high_prices: &[f64],
    low_prices: &[f64],
    close_prices: &[f64],
    price_changes: &[f64],
    transaction_counts: &[i32],
) -> Result<(), AppError> {
    let query = r#"
        INSERT INTO stock_day_all (
            trade_date, stock_code, stock_name,
            trade_volume, trade_amount, open_price,
            high_price, low_price, close_price,
            price_change, transaction_count
        )
        SELECT * FROM UNNEST(
            $1::date[], $2::text[], $3::text[],
            $4::bigint[], $5::bigint[], $6::double precision[],
            $7::double precision[], $8::double precision[], $9::double precision[],
            $10::double precision[], $11::int[]
        )
        ON CONFLICT (trade_date, stock_code) DO NOTHING;
    "#;

    sqlx::query(query)
        .bind(trade_dates)
        .bind(stock_codes)
        .bind(stock_names)
        .bind(trade_volumes)
        .bind(trade_amounts)
        .bind(open_prices)
        .bind(high_prices)
        .bind(low_prices)
        .bind(close_prices)
        .bind(price_changes)
        .bind(transaction_counts)
        .execute(state.get_pool())
        .await?;

    Ok(())
}
