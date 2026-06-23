use crate::{
    errors::{AppError, RequestError},
    repositories::ledger as ledger_repo,
    structs::ledger::{
        CategoryList, CategoryOption, LedgerEntry, LedgerListQuery, LedgerRequest, LedgerSummary,
        SummaryQuery, EXPENSE_CATEGORIES, INCOME_CATEGORIES,
    },
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// 涵蓋「全部」時的預設區間端點（Postgres DATE 合法範圍內）
fn epoch_start() -> NaiveDate {
    NaiveDate::from_ymd_opt(1, 1, 1).unwrap()
}
fn epoch_end() -> NaiveDate {
    NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()
}

/// 驗證 kind / category / amount，非法回 422
fn validate(req: &LedgerRequest) -> Result<(), AppError> {
    let categories = match req.kind.as_str() {
        "income" => INCOME_CATEGORIES,
        "expense" => EXPENSE_CATEGORIES,
        other => {
            return Err(RequestError::UnprocessableContent(format!(
                "kind 必須為 income 或 expense，收到 '{other}'"
            ))
            .into())
        }
    };

    if !categories.iter().any(|(value, _)| *value == req.category) {
        let allowed: Vec<&str> = categories.iter().map(|(value, _)| *value).collect();
        return Err(RequestError::UnprocessableContent(format!(
            "category '{}' 不適用於 {}，可用：{}",
            req.category,
            req.kind,
            allowed.join(", ")
        ))
        .into());
    }

    if req.amount <= Decimal::ZERO {
        return Err(RequestError::UnprocessableContent("amount 必須大於 0".to_string()).into());
    }

    Ok(())
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &LedgerListQuery,
) -> Result<Vec<LedgerEntry>, AppError> {
    let page = crate::structs::pagination::PageQuery {
        page: query.page,
        per_page: query.per_page,
    };
    let (limit, offset) = page.to_limit_offset(50);
    ledger_repo::get_by_member(pool, member_id, query, limit, offset).await
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &LedgerRequest,
) -> Result<LedgerEntry, AppError> {
    validate(req)?;
    ledger_repo::create(pool, member_id, req).await
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
    req: &LedgerRequest,
) -> Result<LedgerEntry, AppError> {
    validate(req)?;
    ledger_repo::update(pool, id, member_id, req).await
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    ledger_repo::delete(pool, id, member_id).await
}

pub async fn summary(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &SummaryQuery,
) -> Result<LedgerSummary, AppError> {
    let from = query.from.unwrap_or_else(epoch_start);
    let to = query.to.unwrap_or_else(epoch_end);

    let (total_income, total_expense) = ledger_repo::totals(pool, member_id, from, to).await?;
    let by_category = ledger_repo::by_category(pool, member_id, from, to).await?;
    let monthly = ledger_repo::monthly(pool, member_id, from, to).await?;

    Ok(LedgerSummary {
        total_income,
        total_expense,
        balance: total_income - total_expense,
        by_category,
        monthly,
    })
}

/// 回傳全站固定分類清單（前端下拉選單用）
pub fn categories() -> CategoryList {
    let map = |items: &[(&str, &str)]| {
        items
            .iter()
            .map(|(value, label)| CategoryOption {
                value: value.to_string(),
                label: label.to_string(),
            })
            .collect()
    };
    CategoryList {
        income: map(INCOME_CATEGORIES),
        expense: map(EXPENSE_CATEGORIES),
    }
}
