use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::stocks as stocks_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::{PageQuery, Paginated, StatusFilter},
        roles::Perm,
        stocks::{
            Conditions, GetStockDayAll, StockBuybackMoreInfo, StockBuybackPeriod, StockChange,
            StockDayAll
        }
    }
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, patch},
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/changes", get(list_stock_changes))
            .route("/changes/{id}/pending", patch(reset_stock_change_pending))
            .route("/day_all", get(stock_day_all))
            .route("/buyback_price_gaps", get(buyback_price_gaps))
            .route("/buyback_periods", get(buyback_periods)),
    )
}

async fn list_stock_changes(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<StatusFilter>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Paginated<StockChange>>, AppError> {
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

async fn stock_day_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<GetStockDayAll>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Paginated<StockDayAll>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    Ok(Json(stocks_service::get_stock_day_all_list(state.get_pool(), payload, limit, offset).await?))
}

async fn buyback_price_gaps(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackMoreInfo>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_active_buyback_prices(state.get_pool()).await?))
}

async fn buyback_periods(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackPeriod>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_stock_buyback_periods(state.get_pool()).await?))
}
