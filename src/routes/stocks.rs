use crate::{
    errors::AppError,
    services::stocks as stocks_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::PageQuery,
        roles::Perm,
        stocks::{
            Conditions, GetStockDayAll, StockBuybackMoreInfo, StockBuybackPeriod,
            StockChangePaginatedResponse, StockClosingPriceResponse, StockDayAll, StockRequest,
        },
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use serde::Deserialize;

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/changes", get(list_stock_changes))
            .route("/changes/{id}/pending", patch(reset_stock_change_pending))
            .route("/closing_price_stats", get(get_closing_price_stats))
            .route("/day_all", get(get_stock_day_all))
            .route("/buyback_price_gaps", get(get_buyback_price_gaps))
            .route("/buyback_periods", get(get_buyback_periods)),
    )
}

#[derive(Deserialize)]
struct StatusFilter {
    status: Option<String>,
}

async fn list_stock_changes(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<StatusFilter>,
    Query(page): Query<PageQuery>,
) -> Result<Json<StockChangePaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    let conditions = Conditions { status: filter.status, limit, offset };
    Ok(Json(stocks_service::get_all_stock_changes(state.get_pool(), conditions).await?))
}

async fn reset_stock_change_pending(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::StockUpdate)?;
    stocks_service::update_one_stock_change_pending(state.get_pool(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_closing_price_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<StockRequest>,
) -> Result<Json<StockClosingPriceResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_closing_price_pair_stats(state.get_pool(), state.get_http_client(), &payload).await?))
}

async fn get_stock_day_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<GetStockDayAll>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Vec<StockDayAll>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    Ok(Json(stocks_service::get_stock_day_all_list(state.get_pool(), payload, limit, offset).await?))
}

async fn get_buyback_price_gaps(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackMoreInfo>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_active_buyback_prices(state.get_pool()).await?))
}

async fn get_buyback_periods(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackPeriod>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_stock_buyback_periods(state.get_pool()).await?))
}
