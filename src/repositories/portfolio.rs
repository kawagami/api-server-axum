use crate::{
    errors::{AppError, RequestError},
    state::AppState,
    structs::portfolio::{PortfolioEntry, PortfolioRequest},
};
use uuid::Uuid;

pub async fn get_by_member(state: &AppState, member_id: i64) -> Result<Vec<PortfolioEntry>, AppError> {
    let rows = sqlx::query_as(
        "SELECT id, member_id, stock_code, buy_date, cost_per_share, shares, created_at, updated_at
         FROM portfolio
         WHERE member_id = $1
         ORDER BY buy_date DESC, created_at DESC",
    )
    .bind(member_id)
    .fetch_all(state.get_pool())
    .await?;
    Ok(rows)
}

pub async fn create(
    state: &AppState,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    let row = sqlx::query_as(
        "INSERT INTO portfolio (member_id, stock_code, buy_date, cost_per_share, shares)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, member_id, stock_code, buy_date, cost_per_share, shares, created_at, updated_at",
    )
    .bind(member_id)
    .bind(&req.stock_code)
    .bind(req.buy_date)
    .bind(req.cost_per_share)
    .bind(req.shares)
    .fetch_one(state.get_pool())
    .await?;
    Ok(row)
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    let row: Option<PortfolioEntry> = sqlx::query_as(
        "UPDATE portfolio
         SET stock_code = $1, buy_date = $2, cost_per_share = $3, shares = $4, updated_at = NOW()
         WHERE id = $5 AND member_id = $6
         RETURNING id, member_id, stock_code, buy_date, cost_per_share, shares, created_at, updated_at",
    )
    .bind(&req.stock_code)
    .bind(req.buy_date)
    .bind(req.cost_per_share)
    .bind(req.shares)
    .bind(id)
    .bind(member_id)
    .fetch_optional(state.get_pool())
    .await?;

    row.ok_or(AppError::RequestError(RequestError::NotFound))
}

pub async fn delete(state: &AppState, id: Uuid, member_id: i64) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM portfolio WHERE id = $1 AND member_id = $2",
    )
    .bind(id)
    .bind(member_id)
    .execute(state.get_pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RequestError(RequestError::NotFound));
    }
    Ok(())
}
