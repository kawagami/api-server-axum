use crate::{
    errors::AppError,
    structs::stocks::NewStockClosingPrice,
};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres, QueryBuilder};

pub async fn upsert_stock_closing_prices(
    pool: &Pool<Postgres>,
    data: &[NewStockClosingPrice],
) -> Result<(), AppError> {
    if data.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();

    let mut qb = QueryBuilder::new(
        "INSERT INTO stock_closing_prices (stock_no, date, close_price, created_at, updated_at) ",
    );

    qb.push_values(data.iter(), |mut b, row| {
        b.push_bind(&row.stock_no)
            .push_bind(row.date)
            .push_bind(row.close_price)
            .push_bind(now)
            .push_bind(now);
    });

    qb.push(" ON CONFLICT (stock_no, date) DO UPDATE SET close_price = EXCLUDED.close_price, updated_at = EXCLUDED.updated_at");
    qb.build().execute(pool).await?;

    Ok(())
}

pub async fn get_stock_closing_prices_by_date_range(
    pool: &Pool<Postgres>,
    stock_no: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<NewStockClosingPrice>, AppError> {
    Ok(sqlx::query_as(
        "SELECT stock_no, date, close_price FROM stock_closing_prices
        WHERE stock_no = $1 AND date BETWEEN $2 AND $3",
    )
    .bind(stock_no)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?)
}
