use crate::{
    errors::AppError,
    middleware::auth,
    services::invoices as invoices_service,
    state::AppState,
    structs::{
        invoices::{Invoice, InvoiceListQuery, InvoiceRequest, NotifyPrefRequest, NotifyPrefResponse},
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
        .route("/notify", patch(set_notify))
        .route("/{id}", get(detail).delete(delete))
        .layer(middleware::from_fn_with_state(state, auth::authorize_member))
}

async fn register(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<InvoiceRequest>,
) -> Result<(StatusCode, Json<Invoice>), AppError> {
    let invoice = invoices_service::register(state.get_pool(), auth_member.member_id, &req).await?;
    Ok((StatusCode::CREATED, Json(invoice)))
}

async fn list(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<Vec<Invoice>>, AppError> {
    Ok(Json(
        invoices_service::list(state.get_pool(), auth_member.member_id, &query).await?,
    ))
}

async fn detail(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Invoice>, AppError> {
    Ok(Json(
        invoices_service::get(state.get_pool(), id, auth_member.member_id).await?,
    ))
}

async fn delete(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    invoices_service::delete(state.get_pool(), id, auth_member.member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_notify(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<NotifyPrefRequest>,
) -> Result<Json<NotifyPrefResponse>, AppError> {
    let enabled = invoices_service::set_notify(state.get_pool(), auth_member.member_id, req.enabled).await?;
    Ok(Json(NotifyPrefResponse { enabled }))
}
