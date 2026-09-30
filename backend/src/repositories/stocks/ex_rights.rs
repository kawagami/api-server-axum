use crate::{errors::AppError, structs::stocks::StockExRight};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{Pool, Postgres, QueryBuilder};

pub async fn upsert_ex_rights(pool: &Pool<Postgres>, data: &[StockExRight]) -> Result<(), AppError> {
    if data.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now().naive_utc();
    let mut qb = QueryBuilder::new(
        "INSERT INTO stock_ex_rights (stock_no, ex_date, close_before, ref_price, created_at, updated_at) ",
    );

    qb.push_values(data.iter(), |mut b, row| {
        b.push_bind(&row.stock_no)
            .push_bind(row.ex_date)
            .push_bind(row.close_before)
            .push_bind(row.ref_price)
            .push_bind(now)
            .push_bind(now);
    });

    qb.push(
        " ON CONFLICT (stock_no, ex_date) DO UPDATE SET \
         close_before = EXCLUDED.close_before, \
         ref_price = EXCLUDED.ref_price, \
         updated_at = EXCLUDED.updated_at",
    );
    qb.build().execute(pool).await?;

    Ok(())
}

/// 記錄「`(stock_no, from_date)` 這段已向 TWSE 確認到 `covered_until`（含）」。
/// **只能在該段除權息已成功寫進 `stock_ex_rights` 之後呼叫** —— 這筆紀錄的意思是
/// 「這段期間 DB 裡沒有的就是真的沒有」，先寫它等於騙自己。
pub async fn upsert_ex_rights_checked(
    pool: &Pool<Postgres>,
    stock_no: &str,
    from_date: NaiveDate,
    covered_until: NaiveDate,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO stock_ex_rights_checked (stock_no, from_date, covered_until, checked_at) \
         VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (stock_no, from_date) DO UPDATE SET covered_until = $3, checked_at = NOW()",
    )
    .bind(stock_no)
    .bind(from_date)
    .bind(covered_until)
    .execute(pool)
    .await?;
    Ok(())
}

/// 回傳 `(covered_until, checked_at)`；沒查過回 None。
pub async fn find_ex_rights_checked(
    pool: &Pool<Postgres>,
    stock_no: &str,
    from_date: NaiveDate,
) -> Result<Option<(NaiveDate, DateTime<Utc>)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT covered_until, checked_at FROM stock_ex_rights_checked \
         WHERE stock_no = $1 AND from_date = $2",
    )
    .bind(stock_no)
    .bind(from_date)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_ex_rights_by_range(
    pool: &Pool<Postgres>,
    stock_no: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<StockExRight>, AppError> {
    let rows = sqlx::query_as(
        "SELECT stock_no, ex_date, close_before, ref_price \
         FROM stock_ex_rights \
         WHERE stock_no = $1 AND ex_date BETWEEN $2 AND $3 \
         ORDER BY ex_date ASC",
    )
    .bind(stock_no)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
