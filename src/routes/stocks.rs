use crate::{
    errors::AppError,
    services::stocks as stocks_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::Perm,
        stocks::{
            Conditions, GetStockDayAll, Pagination, StockBuybackMoreInfo, StockBuybackPeriod,
            StockChangePaginatedResponse, StockChangeId, StockClosingPriceResponse, StockRequest,
            StockDayAll,
        },
    },
};
use axum::{
    extract::{Extension, Query, State},
    routing::{get, patch},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/get_all_stock_changes", get(get_all_stock_changes))
            .route(
                "/update_one_stock_change_pending",
                patch(update_one_stock_change_pending),
            )
            .route(
                "/fetch_stock_closing_price_pair_stats",
                get(fetch_stock_closing_price_pair_stats),
            )
            .route("/get_stock_day_all", get(get_stock_day_all))
            .route(
                "/get_unfinished_buyback_price_gap",
                get(get_unfinished_buyback_price_gap),
            )
            .route(
                "/get_stock_buyback_periods_v2",
                get(get_stock_buyback_periods_v2),
            ),
    )
}

pub async fn get_all_stock_changes(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<Conditions>,
) -> Result<Json<StockChangePaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_all_stock_changes(&state, payload).await?))
}

pub async fn update_one_stock_change_pending(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(payload): Json<StockChangeId>,
) -> Result<Json<()>, AppError> {
    auth_user.require_permission(Perm::StockUpdate)?;
    Ok(Json(
        stocks_service::update_one_stock_change_pending(&state, payload.id).await?,
    ))
}

pub async fn fetch_stock_closing_price_pair_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<StockRequest>,
) -> Result<Json<StockClosingPriceResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_closing_price_pair_stats(&state, &payload).await?))
}

pub async fn get_stock_day_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<GetStockDayAll>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<Vec<StockDayAll>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_stock_day_all_list(&state, payload, pagination).await?))
}

pub async fn get_unfinished_buyback_price_gap(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackMoreInfo>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_active_buyback_prices(&state).await?))
}

pub async fn get_stock_buyback_periods_v2(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackPeriod>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks_service::get_stock_buyback_periods(&state).await?))
}
