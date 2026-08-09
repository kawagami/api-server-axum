use crate::extract::{Json, Path};
use crate::{
    errors::AppError,
    services::portfolio as portfolio_service,
    state::AppState,
    structs::{
        members::AuthenticatedMember,
        portfolio::{HistoryRecord, PortfolioEntry, PortfolioRequest, PortfolioSummaryEntry},
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
    Router
};
use uuid::Uuid;

// 走 super::with_member_auth 而不是直接掛 authorize_member：寫入要進 admin_audit_logs
// （直接掛 auth middleware 會跳過 audit 層，那是 member 操作長年零紀錄的原因）
pub fn new(state: AppState) -> Router<AppState> {
    super::with_member_auth(
        state,
        Router::new()
            .route("/", get(list).post(create))
            .route("/summary", get(summary))
            .route("/{id}", axum::routing::put(update).delete(delete))
            .route("/{id}/history", get(history)),
    )
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
) -> Result<(StatusCode, Json<PortfolioEntry>), AppError> {
    let entry = portfolio_service::create(state.get_pool(), auth_member.member_id, &req).await?;
    Ok((StatusCode::CREATED, Json(entry)))
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
