use crate::{
    errors::AppError,
    middleware::auth,
    services::portfolio as portfolio_service,
    state::AppState,
    structs::{
        members::AuthenticatedMember,
        portfolio::{HistoryRecord, PortfolioEntry, PortfolioRequest, PortfolioSummaryEntry},
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/summary", get(summary))
        .route("/{id}", axum::routing::put(update).delete(delete))
        .route("/{id}/history", get(history))
        .layer(middleware::from_fn_with_state(state, auth::authorize_member))
}

async fn list(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PortfolioEntry>>, AppError> {
    Ok(Json(portfolio_service::get_by_member(state.get_pool(), auth_member.member_id).await?))
}

async fn create(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<PortfolioRequest>,
) -> Result<Json<PortfolioEntry>, AppError> {
    Ok(Json(portfolio_service::create(state.get_pool(), auth_member.member_id, &req).await?))
}

async fn update(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<PortfolioRequest>,
) -> Result<Json<PortfolioEntry>, AppError> {
    Ok(Json(portfolio_service::update(state.get_pool(), id, auth_member.member_id, &req).await?))
}

async fn delete(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    portfolio_service::delete(state.get_pool(), id, auth_member.member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn summary(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PortfolioSummaryEntry>>, AppError> {
    Ok(Json(portfolio_service::get_summary(state.get_pool(), state.get_redis_pool(), state.get_http_client(), auth_member.member_id).await?))
}

async fn history(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<HistoryRecord>>, AppError> {
    Ok(Json(portfolio_service::get_history(state.get_pool(), state.get_redis_pool(), state.get_http_client(), id, auth_member.member_id).await?))
}
