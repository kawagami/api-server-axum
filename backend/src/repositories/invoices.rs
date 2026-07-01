use crate::{
    errors::{AppError, RequestError},
    services::invoice_lottery::PeriodNumbers,
    structs::invoices::{Invoice, InvoiceListQuery, InvoiceRequest, PeriodDraw, WinnerRow},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const COLS: &str = "id, member_id, invoice_number, invoice_date, period, amount, seller_tax_id, \
     source, ledger_entry_id, lottery_checked, prize_tier, notified_at, created_at, updated_at";

/// 登錄一張發票；同 member 同號碼重複（unique 違反）回 409
pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &InvoiceRequest,
    period: &str,
) -> Result<Invoice, AppError> {
    let result = sqlx::query_as(&format!(
        "INSERT INTO invoices
            (member_id, invoice_number, invoice_date, period, amount, seller_tax_id, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING {COLS}"
    ))
    .bind(member_id)
    .bind(&req.invoice_number)
    .bind(req.invoice_date)
    .bind(period)
    .bind(req.amount)
    .bind(&req.seller_tax_id)
    .bind(&req.source)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => Ok(row),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => Err(
            RequestError::Conflict(format!("發票 {} 已登錄過", req.invoice_number)).into(),
        ),
        Err(e) => Err(e.into()),
    }
}

pub async fn link_ledger(
    pool: &Pool<Postgres>,
    id: Uuid,
    ledger_entry_id: Uuid,
) -> Result<Invoice, AppError> {
    let row = sqlx::query_as(&format!(
        "UPDATE invoices SET ledger_entry_id = $1, updated_at = NOW() WHERE id = $2 RETURNING {COLS}"
    ))
    .bind(ledger_entry_id)
    .bind(id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &InvoiceListQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<Invoice>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM invoices
         WHERE member_id = $1
           AND ($2::text IS NULL OR period = $2)
           AND ($3::bool IS NULL OR (prize_tier IS NOT NULL) = $3)
         ORDER BY invoice_date DESC, created_at DESC
         LIMIT $4 OFFSET $5"
    ))
    .bind(member_id)
    .bind(&query.period)
    .bind(query.won)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_for_member(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
) -> Result<Invoice, AppError> {
    let row: Option<Invoice> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM invoices WHERE id = $1 AND member_id = $2"
    ))
    .bind(id)
    .bind(member_id)
    .fetch_optional(pool)
    .await?;
    row.ok_or(AppError::RequestError(RequestError::NotFound))
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM invoices WHERE id = $1 AND member_id = $2")
        .bind(id)
        .bind(member_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::RequestError(RequestError::NotFound));
    }
    Ok(())
}

// ── 通知偏好 ──────────────────────────────────────────────

/// 取 member email（member 必存在；回傳其 email，可能為 null）
pub async fn get_member_email(pool: &Pool<Postgres>, member_id: i64) -> Result<Option<String>, AppError> {
    let row: (Option<String>,) = sqlx::query_as("SELECT email FROM members WHERE id = $1")
        .bind(member_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn set_notify_pref(pool: &Pool<Postgres>, member_id: i64, enabled: bool) -> Result<(), AppError> {
    sqlx::query("UPDATE members SET lottery_notify_enabled = $1 WHERE id = $2")
        .bind(enabled)
        .bind(member_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── 對獎 job 用 ───────────────────────────────────────────

/// upsert 一期的中獎號碼
pub async fn upsert_period_numbers(
    pool: &Pool<Postgres>,
    period: &str,
    nums: &PeriodNumbers,
) -> Result<(), AppError> {
    let mut rows: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = &nums.special {
        rows.push(("special", s));
    }
    if let Some(g) = &nums.grand {
        rows.push(("grand", g));
    }
    for f in &nums.first {
        rows.push(("first", f));
    }
    for a in &nums.additional {
        rows.push(("additional", a));
    }
    for (tier, number) in rows {
        sqlx::query(
            "INSERT INTO invoice_lottery_numbers (period, prize_tier, number)
             VALUES ($1, $2, $3) ON CONFLICT (period, prize_tier, number) DO NOTHING",
        )
        .bind(period)
        .bind(tier)
        .bind(number)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// 有中獎號碼、且尚有未對獎發票的期別
pub async fn periods_pending_check(pool: &Pool<Postgres>) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT DISTINCT i.period FROM invoices i
         WHERE i.lottery_checked = false
           AND EXISTS (SELECT 1 FROM invoice_lottery_numbers n WHERE n.period = i.period)",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// 組出某期的 PeriodNumbers
pub async fn load_period_numbers(pool: &Pool<Postgres>, period: &str) -> Result<PeriodNumbers, AppError> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT prize_tier, number FROM invoice_lottery_numbers WHERE period = $1")
            .bind(period)
            .fetch_all(pool)
            .await?;
    let mut n = PeriodNumbers::default();
    for (tier, number) in rows {
        match tier.as_str() {
            "special" => n.special = Some(number),
            "grand" => n.grand = Some(number),
            "first" => n.first.push(number),
            "additional" => n.additional.push(number),
            _ => {}
        }
    }
    Ok(n)
}

/// 近期各期中獎號碼（前端展示用），一期一筆；rows 依 period DESC 排序，同期相鄰後於 Rust 分組
pub async fn recent_period_draws(
    pool: &Pool<Postgres>,
    period: Option<&str>,
    limit: i64,
) -> Result<Vec<PeriodDraw>, AppError> {
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT period, prize_tier, number FROM invoice_lottery_numbers
         WHERE period IN (
             SELECT period FROM invoice_lottery_numbers
             WHERE ($1::text IS NULL OR period = $1)
             GROUP BY period ORDER BY period DESC LIMIT $2
         )
         ORDER BY period DESC, prize_tier, number",
    )
    .bind(period)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut out: Vec<PeriodDraw> = Vec::new();
    for (p, tier, number) in rows {
        if out.last().map(|e| e.period != p).unwrap_or(true) {
            out.push(PeriodDraw {
                period: p,
                special: None,
                grand: None,
                first: Vec::new(),
                additional: Vec::new(),
            });
        }
        let entry = out.last_mut().unwrap();
        match tier.as_str() {
            "special" => entry.special = Some(number),
            "grand" => entry.grand = Some(number),
            "first" => entry.first.push(number),
            "additional" => entry.additional.push(number),
            _ => {}
        }
    }
    Ok(out)
}

/// 某期未對獎的發票 (id, 號碼)
pub async fn unchecked_by_period(
    pool: &Pool<Postgres>,
    period: &str,
) -> Result<Vec<(Uuid, String)>, AppError> {
    let rows = sqlx::query_as(
        "SELECT id, invoice_number FROM invoices WHERE period = $1 AND lottery_checked = false",
    )
    .bind(period)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_checked(
    pool: &Pool<Postgres>,
    id: Uuid,
    prize_tier: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE invoices SET lottery_checked = true, prize_tier = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(prize_tier)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// admin 改號碼後讓某期重新對獎
pub async fn reset_period_check(pool: &Pool<Postgres>, period: &str) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE invoices SET lottery_checked = false, prize_tier = NULL, updated_at = NOW() WHERE period = $1",
    )
    .bind(period)
    .execute(pool)
    .await?;
    Ok(())
}

/// 待通知的中獎發票（已開啟通知、有 email、尚未寄過）
pub async fn winners_to_notify(pool: &Pool<Postgres>) -> Result<Vec<WinnerRow>, AppError> {
    let rows = sqlx::query_as(
        "SELECT i.id, i.member_id, i.invoice_number, i.period, i.prize_tier, m.email
         FROM invoices i JOIN members m ON m.id = i.member_id
         WHERE i.prize_tier IS NOT NULL
           AND i.notified_at IS NULL
           AND m.lottery_notify_enabled = true
           AND m.email IS NOT NULL
         ORDER BY i.member_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_notified(pool: &Pool<Postgres>, ids: &[Uuid]) -> Result<(), AppError> {
    sqlx::query("UPDATE invoices SET notified_at = NOW() WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(())
}
