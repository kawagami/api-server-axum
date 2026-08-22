use crate::{errors::AppError, structs::stocks::{StockDayAll, StockDayAllInsertRow}};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

/// list 與 count 共用的 WHERE —— 兩邊漂移會讓 total 與實際筆數對不上
fn push_day_all_filter(
    builder: &mut QueryBuilder<'_, Postgres>,
    stock_code: &Option<String>,
    trade_date: &Option<NaiveDate>,
) {
    if stock_code.is_none() && trade_date.is_none() {
        return;
    }
    builder.push(" WHERE ");

    let mut has_where = false;
    if let Some(code) = stock_code {
        builder.push("stock_code = ").push_bind(code.clone());
        has_where = true;
    }
    if let Some(date) = trade_date {
        if has_where {
            builder.push(" AND ");
        }
        builder.push("trade_date = ").push_bind(*date);
    }
}

pub async fn get_stock_day_all(
    pool: &Pool<Postgres>,
    stock_code: &Option<String>,
    trade_date: &Option<NaiveDate>,
    limit: i64,
    offset: i64,
) -> Result<Vec<StockDayAll>, AppError> {
    let mut builder = QueryBuilder::new("SELECT * FROM stock_day_all");
    push_day_all_filter(&mut builder, stock_code, trade_date);

    builder.push(" ORDER BY trade_date DESC, stock_code ASC");
    builder.push(" LIMIT ").push_bind(limit);
    builder.push(" OFFSET ").push_bind(offset);

    Ok(builder.build_query_as::<StockDayAll>().fetch_all(pool).await?)
}

pub async fn count_stock_day_all(
    pool: &Pool<Postgres>,
    stock_code: &Option<String>,
    trade_date: &Option<NaiveDate>,
) -> Result<i64, AppError> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM stock_day_all");
    push_day_all_filter(&mut builder, stock_code, trade_date);

    let (total,): (i64,) = builder.build_query_as().fetch_one(pool).await?;
    Ok(total)
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

/// 一次取多檔的最新股名。
///
/// 存在的理由是 `services::portfolio::get_summary`：原本每檔持股各發一次
/// `get_stock_name_by_code`，N 檔就是 N 個查詢跟同一次請求的其他查詢搶 PG 連線。
/// `DISTINCT ON` 走的是既有的 `(stock_code, trade_date DESC)` 索引，一次搞定。
///
/// 回傳的 map **只含查得到的 code**（沒行情資料的檔就是缺鍵，呼叫端當 None 處理）。
pub async fn get_stock_names_by_codes(
    pool: &Pool<Postgres>,
    codes: &[String],
) -> Result<HashMap<String, String>, AppError> {
    if codes.is_empty() {
        return Ok(HashMap::new());
    }

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT DISTINCT ON (stock_code) stock_code, stock_name
         FROM stock_day_all
         WHERE stock_code = ANY($1)
         ORDER BY stock_code, trade_date DESC",
    )
    .bind(codes)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().collect())
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
