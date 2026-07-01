use crate::{
    errors::{AppError, RequestError},
    repositories::{invoices as invoices_repo, ledger as ledger_repo},
    services::invoice_lottery::{period_of_date, PeriodNumbers},
    structs::{
        invoices::{AdminLotteryNumbersRequest, Invoice, InvoiceListQuery, InvoiceRequest},
        ledger::EXPENSE_CATEGORIES,
    },
};
use regex::Regex;
use sqlx::{Pool, Postgres};
use std::sync::OnceLock;
use uuid::Uuid;

const SOURCES: &[&str] = &["qr", "barcode", "manual"];

fn invoice_number_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{2}\d{8}$").unwrap())
}

fn unprocessable(msg: &str) -> AppError {
    RequestError::UnprocessableContent(msg.to_string()).into()
}

/// 登錄發票（前門）；record_as_expense 時一併建 ledger 並連結
pub async fn register(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &InvoiceRequest,
) -> Result<Invoice, AppError> {
    if !invoice_number_re().is_match(&req.invoice_number) {
        return Err(unprocessable("invoice_number 格式須為 2 大寫英文 + 8 數字，如 AB12345678"));
    }
    if !SOURCES.contains(&req.source.as_str()) {
        return Err(unprocessable("source 必須為 qr / barcode / manual"));
    }

    let period = period_of_date(req.invoice_date);
    let invoice = invoices_repo::create(pool, member_id, req, &period).await?;

    if !req.record_as_expense {
        return Ok(invoice);
    }

    // 同時記成一筆支出
    let amount = req
        .amount
        .ok_or_else(|| unprocessable("record_as_expense 為 true 時必須提供 amount"))?;
    let category = req.category.clone().unwrap_or_else(|| "other".to_string());
    if !EXPENSE_CATEGORIES.iter().any(|(v, _)| *v == category) {
        return Err(unprocessable("category 不是合法的支出分類"));
    }

    let entry = ledger_repo::create_from_invoice(
        pool,
        member_id,
        amount,
        &category,
        req.note.as_deref(),
        req.invoice_date,
        &req.invoice_number,
        req.seller_tax_id.as_deref(),
    )
    .await?;

    invoices_repo::link_ledger(pool, invoice.id, entry.id).await
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &InvoiceListQuery,
) -> Result<Vec<Invoice>, AppError> {
    let page = crate::structs::pagination::PageQuery {
        page: query.page,
        per_page: query.per_page,
    };
    let (limit, offset) = page.to_limit_offset(50);
    invoices_repo::list(pool, member_id, query, limit, offset).await
}

pub async fn get(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<Invoice, AppError> {
    invoices_repo::get_for_member(pool, id, member_id).await
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    invoices_repo::delete(pool, id, member_id).await
}

/// 開關中獎 email 通知；開啟須有 email
pub async fn set_notify(pool: &Pool<Postgres>, member_id: i64, enabled: bool) -> Result<bool, AppError> {
    if enabled {
        let email = invoices_repo::get_member_email(pool, member_id).await?;
        if email.filter(|e| !e.is_empty()).is_none() {
            return Err(unprocessable("此帳號未綁定 email，無法開啟中獎通知"));
        }
    }
    invoices_repo::set_notify_pref(pool, member_id, enabled).await?;
    Ok(enabled)
}

/// admin 手動補某期中獎號碼，並讓該期重新對獎
pub async fn admin_set_numbers(
    pool: &Pool<Postgres>,
    req: &AdminLotteryNumbersRequest,
) -> Result<usize, AppError> {
    let nums = PeriodNumbers {
        special: req.special.clone(),
        grand: req.grand.clone(),
        first: req.first.clone(),
        additional: req.additional.clone(),
    };
    let count = nums.special.is_some() as usize
        + nums.grand.is_some() as usize
        + nums.first.len()
        + nums.additional.len();

    invoices_repo::upsert_period_numbers(pool, &req.period, &nums).await?;
    invoices_repo::reset_period_check(pool, &req.period).await?;
    Ok(count)
}
