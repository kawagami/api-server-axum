use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::invoices as invoices_service,
    state::AppState,
    structs::{
        invoices::{DrawListQuery, Invoice, InvoiceListQuery, InvoiceRequest, PeriodDraw},
        members::AuthenticatedMember,
        notify::{NotifyPrefRequest, NotifyPrefResponse},
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
    Json(req): Json<InvoiceRequest>,
) -> Result<(StatusCode, Json<Invoice>), AppError> {
    let invoice = invoices_service::register(state.get_pool(), auth_member.member_id, &req).await?;
    Ok((StatusCode::CREATED, Json(invoice)))
}

async fn list(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<InvoiceListQuery>,
) -> Result<Json<Paginated<Invoice>>, AppError> {
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

async fn draws(
    Extension(_auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<DrawListQuery>,
) -> Result<Json<Vec<PeriodDraw>>, AppError> {
    Ok(Json(invoices_service::draws(state.get_pool(), &query).await?))
}

async fn set_notify(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<NotifyPrefRequest>,
) -> Result<Json<NotifyPrefResponse>, AppError> {
    let enabled = invoices_service::set_notify(state.get_pool(), auth_member.member_id, req.enabled).await?;
    Ok(Json(NotifyPrefResponse { enabled }))
}
