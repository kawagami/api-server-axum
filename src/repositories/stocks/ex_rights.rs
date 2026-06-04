use crate::{errors::AppError, state::AppState, structs::stocks::StockExRight};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::QueryBuilder;

pub async fn upsert_ex_rights(state: &AppState, data: &[StockExRight]) -> Result<(), AppError> {
    if data.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();
    let mut qb = QueryBuilder::new(
        "INSERT INTO stock_ex_rights (stock_no, ex_date, close_before, cash_div, stock_rate, created_at, updated_at) ",
    );

    qb.push_values(data.iter(), |mut b, row| {
        b.push_bind(&row.stock_no)
            .push_bind(row.ex_date)
            .push_bind(row.close_before)
            .push_bind(row.cash_div)
            .push_bind(row.stock_rate)
            .push_bind(now)
            .push_bind(now);
    });

    qb.push(
        " ON CONFLICT (stock_no, ex_date) DO UPDATE SET \
         close_before = EXCLUDED.close_before, \
         cash_div = EXCLUDED.cash_div, \
         stock_rate = EXCLUDED.stock_rate, \
         updated_at = EXCLUDED.updated_at",
    );
    qb.build().execute(state.get_pool()).await?;

    Ok(())
}

pub async fn upsert_ex_rights_checked(
    state: &AppState,
    stock_no: &str,
    from_date: NaiveDate,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO stock_ex_rights_checked (stock_no, from_date, checked_at) \
         VALUES ($1, $2, NOW()) \
         ON CONFLICT (stock_no, from_date) DO UPDATE SET checked_at = NOW()",
    )
    .bind(stock_no)
    .bind(from_date)
    .execute(state.get_pool())
    .await?;
    Ok(())
}

pub async fn find_ex_rights_checked(
    state: &AppState,
    stock_no: &str,
    from_date: NaiveDate,
) -> Result<Option<DateTime<Utc>>, AppError> {
    let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
        "SELECT checked_at FROM stock_ex_rights_checked \
         WHERE stock_no = $1 AND from_date = $2",
    )
    .bind(stock_no)
    .bind(from_date)
    .fetch_optional(state.get_pool())
    .await?;
    Ok(row.map(|(t,)| t))
}

pub async fn get_ex_rights_by_range(
    state: &AppState,
    stock_no: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<StockExRight>, AppError> {
    let rows = sqlx::query_as(
        "SELECT stock_no, ex_date, close_before, cash_div, stock_rate \
         FROM stock_ex_rights \
         WHERE stock_no = $1 AND ex_date BETWEEN $2 AND $3 \
         ORDER BY ex_date ASC",
    )
    .bind(stock_no)
    .bind(from)
    .bind(to)
    .fetch_all(state.get_pool())
    .await?;
    Ok(rows)
}
