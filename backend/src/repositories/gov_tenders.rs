use crate::{
    errors::AppError,
    structs::gov_tenders::{GovTender, GovTenderListQuery, NewGovTender},
};
use sqlx::{Pool, Postgres};

const COLS: &str = "id, filename, date, tender_type, title, category, unit_id, unit_name, \
     job_number, companies, keyword, detail_url, notified_at, created_at";

/// 寫入一筆標案；filename 已存在則跳過並回 None，新寫入回 Some
pub async fn insert_ignore(
    pool: &Pool<Postgres>,
    t: &NewGovTender,
) -> Result<Option<GovTender>, AppError> {
    let row = sqlx::query_as(&format!(
        "INSERT INTO gov_tenders
            (filename, date, tender_type, title, category, unit_id, unit_name,
             job_number, companies, keyword, detail_url)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (filename) DO NOTHING
         RETURNING {COLS}"
    ))
    .bind(&t.filename)
    .bind(t.date)
    .bind(&t.tender_type)
    .bind(&t.title)
    .bind(&t.category)
    .bind(&t.unit_id)
    .bind(&t.unit_name)
    .bind(&t.job_number)
    .bind(serde_json::json!(t.companies))
    .bind(&t.keyword)
    .bind(&t.detail_url)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn has_any(pool: &Pool<Postgres>) -> Result<bool, AppError> {
    let (exists,): (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM gov_tenders)")
        .fetch_one(pool)
        .await?;
    Ok(exists)
}

/// 尚未寄過新標案通知的資料（舊到新）
pub async fn unnotified(pool: &Pool<Postgres>) -> Result<Vec<GovTender>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM gov_tenders WHERE notified_at IS NULL ORDER BY date, id"
    ))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_notified(pool: &Pool<Postgres>, ids: &[i64]) -> Result<(), AppError> {
    if ids.is_empty() {
        return Ok(());
    }
    sqlx::query("UPDATE gov_tenders SET notified_at = NOW() WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list(
    pool: &Pool<Postgres>,
    query: &GovTenderListQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<GovTender>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM gov_tenders
         WHERE ($1::text IS NULL OR keyword = $1)
           AND ($2::text IS NULL OR tender_type = $2)
           AND ($3::text IS NULL OR title ILIKE '%' || $3 || '%' OR unit_name ILIKE '%' || $3 || '%')
         ORDER BY date DESC, id DESC
         LIMIT $4 OFFSET $5"
    ))
    .bind(&query.keyword)
    .bind(&query.tender_type)
    .bind(&query.q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 所有出現過的標案類型（去重、字母排序）
pub async fn distinct_types(pool: &Pool<Postgres>) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT tender_type FROM gov_tenders ORDER BY tender_type")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(t,)| t).collect())
}

pub async fn count(pool: &Pool<Postgres>, query: &GovTenderListQuery) -> Result<i64, AppError> {
    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM gov_tenders
         WHERE ($1::text IS NULL OR keyword = $1)
           AND ($2::text IS NULL OR tender_type = $2)
           AND ($3::text IS NULL OR title ILIKE '%' || $3 || '%' OR unit_name ILIKE '%' || $3 || '%')",
    )
    .bind(&query.keyword)
    .bind(&query.tender_type)
    .bind(&query.q)
    .fetch_one(pool)
    .await?;
    Ok(total)
}
