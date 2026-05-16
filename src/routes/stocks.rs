use crate::{
    errors::AppError,
    middleware::auth,
    repositories::stocks,
    services::stocks::{fetch_stock_price_for_date, round_to_n_decimal},
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::Perm,
        stocks::{
            Conditions, GetStockDayAll, Pagination, StockBuybackMoreInfo, StockBuybackPeriod,
            StockChangePaginatedResponse, StockChangeId, StockClosingPriceResponse, StockRequest,
            StockStats,
        },
    },
};
use axum::{
    extract::{Extension, Query, State},
    middleware,
    routing::{get, patch},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
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
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ))
}

pub async fn get_all_stock_changes(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<Conditions>,
) -> Result<Json<StockChangePaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks::get_all_stock_changes(&state, payload).await?))
}

pub async fn update_one_stock_change_pending(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(payload): Json<StockChangeId>,
) -> Result<Json<()>, AppError> {
    auth_user.require_permission(Perm::StockUpdate)?;
    Ok(Json(
        stocks::update_one_stock_change_pending(&state, payload.id).await?,
    ))
}

pub async fn fetch_stock_closing_price_pair_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<StockRequest>,
) -> Result<Json<StockClosingPriceResponse>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let (start_price, end_price) = tokio::try_join!(
        fetch_stock_price_for_date(&state, &payload.stock_no, &payload.start_date),
        fetch_stock_price_for_date(&state, &payload.stock_no, &payload.end_date)
    )?;

    let price_diff = round_to_n_decimal(end_price.close_price - start_price.close_price, 2);
    let raw_percent_change = if start_price.close_price != 0.0 {
        (price_diff / start_price.close_price) * 100.0
    } else {
        0.0
    };
    let percent_change = round_to_n_decimal(raw_percent_change, 2);
    let is_increase = price_diff > 0.0;

    let day_span = (end_price.date - start_price.date).num_days();

    let stats = StockStats {
        price_diff,
        percent_change,
        is_increase,
        day_span,
    };

    let response = StockClosingPriceResponse {
        prices: (start_price, end_price),
        stats,
    };

    Ok(Json(response))
}

pub async fn get_stock_day_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(payload): Query<GetStockDayAll>,
    Query(pagination): Query<Pagination>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let response = stocks::get_stock_day_all(
        &state,
        payload.stock_code,
        payload.trade_date,
        pagination.limit,
        pagination.offset,
    )
    .await?;

    Ok(Json(response))
}

pub async fn get_unfinished_buyback_price_gap(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackMoreInfo>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    Ok(Json(stocks::get_active_buyback_prices(&state).await?))
}

pub async fn get_stock_buyback_periods_v2(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<StockBuybackPeriod>>, AppError> {
    auth_user.require_permission(Perm::StockRead)?;
    let data = stocks::get_stock_buyback_periods(&state).await?;

    Ok(Json(data))
}
