use crate::{
    errors::AppError,
    repositories::portfolio as portfolio_repo,
    structs::portfolio::{PortfolioEntry, PortfolioRequest},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub async fn get_by_member(pool: &Pool<Postgres>, member_id: i64) -> Result<Vec<PortfolioEntry>, AppError> {
    portfolio_repo::get_by_member(pool, member_id).await
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    req.validate(crate::utils::date::taipei_today())
        .map_err(crate::errors::RequestError::UnprocessableContent)?;
    portfolio_repo::create(pool, member_id, req).await
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
    req: &PortfolioRequest,
) -> Result<PortfolioEntry, AppError> {
    req.validate(crate::utils::date::taipei_today())
        .map_err(crate::errors::RequestError::UnprocessableContent)?;
    portfolio_repo::update(pool, id, member_id, req).await
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    portfolio_repo::delete(pool, id, member_id).await
}
