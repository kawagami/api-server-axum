use crate::{
    errors::{AppError, RequestError},
    structs::ledger::{CategorySum, LedgerEntry, LedgerListQuery, LedgerRequest, MonthlySum},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const COLS: &str =
    "id, member_id, kind, amount, category, note, occurred_at, created_at, updated_at";

pub async fn get_by_member(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &LedgerListQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<LedgerEntry>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM ledger_entries
         WHERE member_id = $1
           AND ($2::text IS NULL OR kind = $2)
           AND ($3::text IS NULL OR category = $3)
           AND ($4::date IS NULL OR occurred_at >= $4)
           AND ($5::date IS NULL OR occurred_at <= $5)
         ORDER BY occurred_at DESC, created_at DESC
         LIMIT $6 OFFSET $7"
    ))
    .bind(member_id)
    .bind(&query.kind)
    .bind(&query.category)
    .bind(query.from)
    .bind(query.to)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &LedgerRequest,
) -> Result<LedgerEntry, AppError> {
    let row = sqlx::query_as(&format!(
        "INSERT INTO ledger_entries (member_id, kind, amount, category, note, occurred_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING {COLS}"
    ))
    .bind(member_id)
    .bind(&req.kind)
    .bind(req.amount)
    .bind(&req.category)
    .bind(&req.note)
    .bind(req.occurred_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
    req: &LedgerRequest,
) -> Result<LedgerEntry, AppError> {
    let row: Option<LedgerEntry> = sqlx::query_as(&format!(
        "UPDATE ledger_entries
         SET kind = $1, amount = $2, category = $3, note = $4, occurred_at = $5, updated_at = NOW()
         WHERE id = $6 AND member_id = $7
         RETURNING {COLS}"
    ))
    .bind(&req.kind)
    .bind(req.amount)
    .bind(&req.category)
    .bind(&req.note)
    .bind(req.occurred_at)
    .bind(id)
    .bind(member_id)
    .fetch_optional(pool)
    .await?;

    row.ok_or(AppError::RequestError(RequestError::NotFound))
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM ledger_entries WHERE id = $1 AND member_id = $2")
        .bind(id)
        .bind(member_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RequestError(RequestError::NotFound));
    }
    Ok(())
}

/// 區間內的總收入 / 總支出
pub async fn totals(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(Decimal, Decimal), AppError> {
    let row: (Decimal, Decimal) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(amount) FILTER (WHERE kind = 'income'), 0) AS total_income,
            COALESCE(SUM(amount) FILTER (WHERE kind = 'expense'), 0) AS total_expense
         FROM ledger_entries
         WHERE member_id = $1 AND occurred_at BETWEEN $2 AND $3",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// 區間內依 kind + category 分組加總
pub async fn by_category(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CategorySum>, AppError> {
    let rows = sqlx::query_as(
        "SELECT kind, category, COALESCE(SUM(amount), 0) AS total
         FROM ledger_entries
         WHERE member_id = $1 AND occurred_at BETWEEN $2 AND $3
         GROUP BY kind, category
         ORDER BY kind, total DESC",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 區間內每月收入 / 支出趨勢
pub async fn monthly(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<MonthlySum>, AppError> {
    let rows = sqlx::query_as(
        "SELECT
            to_char(occurred_at, 'YYYY-MM') AS month,
            COALESCE(SUM(amount) FILTER (WHERE kind = 'income'), 0) AS income,
            COALESCE(SUM(amount) FILTER (WHERE kind = 'expense'), 0) AS expense
         FROM ledger_entries
         WHERE member_id = $1 AND occurred_at BETWEEN $2 AND $3
         GROUP BY month
         ORDER BY month",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
