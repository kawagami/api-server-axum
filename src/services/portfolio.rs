use crate::{
    errors::AppError,
    repositories::portfolio as portfolio_repo,
    state::AppState,
    structs::portfolio::{PortfolioEntry, PortfolioRequest},
};
use uuid::Uuid;

pub async fn get_by_member(state: &AppState, member_id: i64) -> Result<Vec<PortfolioEntry>, AppError> {
    portfolio_repo::get_by_member(state, member_id).await
}

pub async fn create(
    state: &AppState,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    portfolio_repo::create(state, member_id, req).await
}

pub async fn update(
    state: &AppState,
    id: Uuid,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    portfolio_repo::update(state, id, member_id, req).await
}

pub async fn delete(state: &AppState, id: Uuid, member_id: i64) -> Result<(), AppError> {
    portfolio_repo::delete(state, id, member_id).await
}
