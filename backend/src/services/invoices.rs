use crate::{
    errors::{AppError, RequestError},
    repositories::{invoices as invoices_repo, ledger as ledger_repo},
    services::invoice_lottery::{period_of_date, PeriodNumbers},
    structs::{
        invoices::{
            AdminLotteryNumbersRequest, DrawListQuery, Invoice, InvoiceListQuery, InvoiceRequest,
            PeriodDraw,
        },
        ledger::EXPENSE_CATEGORIES,
    },
};
use regex::Regex;
use rust_decimal::Decimal;
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

/// 取出並驗證「順便記一筆支出」需要的欄位；`record_as_expense=false` 時回 None。
///
/// 抽成純函式是為了讓這段**保證跑在任何寫入之前**——先前的版本在發票已 INSERT
/// 之後才驗 amount/category，回 422 的同時發票已落地，使用者重試就撞 unique
/// violation 變 409「已登錄過」，那張發票從此既登不進去也拿不到帳目。
fn expense_fields(req: &InvoiceRequest) -> Result<Option<(Decimal, String)>, AppError> {
    if !req.record_as_expense {
        return Ok(None);
    }
    let amount = req
        .amount
        .ok_or_else(|| unprocessable("record_as_expense 為 true 時必須提供 amount"))?;
    let category = req.category.clone().unwrap_or_else(|| "other".to_string());
    if !EXPENSE_CATEGORIES.iter().any(|(v, _)| *v == category) {
        return Err(unprocessable("category 不是合法的支出分類"));
    }
    Ok(Some((amount, category)))
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

    let expense = expense_fields(req)?;
    let period = period_of_date(req.invoice_date);

    // 三次寫入（invoices → ledger_entries → 回寫 ledger_entry_id）包同一 transaction：
    // 中途失敗若各自 commit，會留下孤兒 ledger 支出 + ledger_entry_id 為 NULL 的發票，
    // 而重試又被 unique violation 擋成 409，等於永久卡死。
    let mut tx = pool.begin().await?;
    let invoice = invoices_repo::create_in_tx(&mut tx, member_id, req, &period).await?;

    let result = match expense {
        None => invoice,
        Some((amount, category)) => {
            let entry = ledger_repo::create_from_invoice_in_tx(
                &mut tx,
                member_id,
                amount,
                &category,
                req.note.as_deref(),
                req.invoice_date,
                &req.invoice_number,
                req.seller_tax_id.as_deref(),
            )
            .await?;
            invoices_repo::link_ledger_in_tx(&mut tx, invoice.id, entry.id).await?
        }
    };

    tx.commit().await?;
    Ok(result)
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

/// 近期各期中獎號碼（前端展示；不限個人發票）
pub async fn draws(pool: &Pool<Postgres>, query: &DrawListQuery) -> Result<Vec<PeriodDraw>, AppError> {
    let limit = query.limit.unwrap_or(6).clamp(1, 24);
    invoices_repo::recent_period_draws(pool, query.period.as_deref(), limit).await
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

    // 改號碼 + 讓該期重新對獎必須同生同死（理由見 reset_period_check_in_tx 的註解）
    let mut tx = pool.begin().await?;
    invoices_repo::upsert_period_numbers_in_tx(&mut tx, &req.period, &nums).await?;
    invoices_repo::reset_period_check_in_tx(&mut tx, &req.period).await?;
    tx.commit().await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn req(record_as_expense: bool, amount: Option<i64>, category: Option<&str>) -> InvoiceRequest {
        InvoiceRequest {
            invoice_number: "AB12345678".to_string(),
            invoice_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            amount: amount.map(Decimal::from),
            seller_tax_id: None,
            source: "manual".to_string(),
            record_as_expense,
            category: category.map(str::to_string),
            note: None,
        }
    }

    #[test]
    fn no_expense_requested_needs_no_amount() {
        assert!(expense_fields(&req(false, None, None)).unwrap().is_none());
    }

    #[test]
    fn expense_defaults_category_to_other() {
        let (amount, category) = expense_fields(&req(true, Some(120), None))
            .unwrap()
            .expect("record_as_expense=true 應回 Some");
        assert_eq!(amount, Decimal::from(120));
        assert_eq!(category, "other");
    }

    /// 這兩條守的是「驗證必須早於任何寫入」：只要 expense_fields 先擋下來，
    /// 發票就不會落地，使用者才有重試的機會（否則重試會撞 unique 變 409 永久卡死）。
    #[test]
    fn expense_without_amount_is_rejected() {
        assert!(expense_fields(&req(true, None, Some("food"))).is_err());
    }

    #[test]
    fn expense_with_unknown_category_is_rejected() {
        assert!(expense_fields(&req(true, Some(50), Some("not_a_category"))).is_err());
    }
}
