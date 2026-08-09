use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::lotto_tickets as lotto_service,
    state::AppState,
    structs::{
        lotto::{
            Draw, DrawListQuery, NotifyPrefRequest, NotifyPrefResponse, Ticket, TicketBatchRequest,
            TicketListQuery,
        },
        members::AuthenticatedMember,
        pagination::Paginated,
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, patch},
    Router
};
use uuid::Uuid;

// 走 super::with_member_auth：寫入要進 admin_audit_logs（見 routes.rs 的說明）
pub fn new(state: AppState) -> Router<AppState> {
    super::with_member_auth(
        state,
        Router::new()
            .route("/", get(list).post(register))
            .route("/draws", get(draws))
            .route("/notify", patch(set_notify))
            .route("/{id}", get(detail).delete(delete)),
    )
}

async fn register(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<TicketBatchRequest>,
) -> Result<(StatusCode, Json<Vec<Ticket>>), AppError> {
    let tickets = lotto_service::register(state.get_pool(), auth_member.member_id, &req).await?;
    Ok((StatusCode::CREATED, Json(tickets)))
}

async fn list(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<TicketListQuery>,
) -> Result<Json<Paginated<Ticket>>, AppError> {
    Ok(Json(
        lotto_service::list(state.get_pool(), auth_member.member_id, &query).await?,
    ))
}

async fn detail(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Ticket>, AppError> {
    Ok(Json(
        lotto_service::get(state.get_pool(), id, auth_member.member_id).await?,
    ))
}

async fn delete(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    lotto_service::delete(state.get_pool(), id, auth_member.member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn draws(
    Extension(_auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<DrawListQuery>,
) -> Result<Json<Vec<Draw>>, AppError> {
    Ok(Json(lotto_service::draws(state.get_pool(), &query).await?))
}

async fn set_notify(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<NotifyPrefRequest>,
) -> Result<Json<NotifyPrefResponse>, AppError> {
    let enabled =
        lotto_service::set_notify(state.get_pool(), auth_member.member_id, req.enabled).await?;
    Ok(Json(NotifyPrefResponse { enabled }))
}
