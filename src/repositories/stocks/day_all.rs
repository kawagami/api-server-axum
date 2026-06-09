use crate::{errors::AppError, structs::stocks::{StockDayAll, StockDayAllInsertRow}};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres, QueryBuilder};

pub async fn get_stock_day_all(
    pool: &Pool<Postgres>,
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

    Ok(builder.build_query_as::<StockDayAll>().fetch_all(pool).await?)
}

pub async fn get_stock_name_by_code(
    pool: &Pool<Postgres>,
    stock_code: &str,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT stock_name FROM stock_day_all WHERE stock_code = $1 ORDER BY trade_date DESC LIMIT 1",
    )
    .bind(stock_code)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(name,)| name))
}

pub async fn insert_stock_day_all_batch(
    pool: &Pool<Postgres>,
    rows: &[StockDayAllInsertRow],
) -> Result<(), AppError> {
    if rows.is_empty() {
        return Ok(());
    }

    let trade_dates: Vec<NaiveDate> = rows.iter().map(|r| r.trade_date).collect();
    let stock_codes: Vec<&str> = rows.iter().map(|r| r.stock_code.as_str()).collect();
    let stock_names: Vec<&str> = rows.iter().map(|r| r.stock_name.as_str()).collect();
    let trade_volumes: Vec<i64> = rows.iter().map(|r| r.trade_volume).collect();
    let trade_amounts: Vec<i64> = rows.iter().map(|r| r.trade_amount).collect();
    let open_prices: Vec<Decimal> = rows.iter().map(|r| r.open_price).collect();
    let high_prices: Vec<Decimal> = rows.iter().map(|r| r.high_price).collect();
    let low_prices: Vec<Decimal> = rows.iter().map(|r| r.low_price).collect();
    let close_prices: Vec<Decimal> = rows.iter().map(|r| r.close_price).collect();
    let price_changes: Vec<Decimal> = rows.iter().map(|r| r.price_change).collect();
    let transaction_counts: Vec<i32> = rows.iter().map(|r| r.transaction_count).collect();

    let query = r#"
        INSERT INTO stock_day_all (
            trade_date, stock_code, stock_name,
            trade_volume, trade_amount, open_price,
            high_price, low_price, close_price,
            price_change, transaction_count
        )
        SELECT * FROM UNNEST(
            $1::date[], $2::text[], $3::text[],
            $4::bigint[], $5::bigint[], $6::numeric[],
            $7::numeric[], $8::numeric[], $9::numeric[],
            $10::numeric[], $11::int[]
        )
        ON CONFLICT (trade_date, stock_code) DO NOTHING;
    "#;

    sqlx::query(query)
        .bind(&trade_dates)
        .bind(&stock_codes)
        .bind(&stock_names)
        .bind(&trade_volumes)
        .bind(&trade_amounts)
        .bind(&open_prices)
        .bind(&high_prices)
        .bind(&low_prices)
        .bind(&close_prices)
        .bind(&price_changes)
        .bind(&transaction_counts)
        .execute(pool)
        .await?;

    Ok(())
}
