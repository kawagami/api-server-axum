use crate::{
    errors::AppError,
    middleware::auth,
    services::lotto_tickets as lotto_service,
    state::AppState,
    structs::{
        lotto::{
            Draw, DrawListQuery, NotifyPrefRequest, NotifyPrefResponse, Ticket, TicketBatchRequest,
            TicketListQuery,
        },
        members::AuthenticatedMember,
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, patch},
    Json, Router,
};
use uuid::Uuid;

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(register))
        .route("/draws", get(draws))
        .route("/notify", patch(set_notify))
        .route("/{id}", get(detail).delete(delete))
        .layer(middleware::from_fn_with_state(state, auth::authorize_member))
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
) -> Result<Json<Vec<Ticket>>, AppError> {
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
