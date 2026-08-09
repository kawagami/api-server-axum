use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::ledger as ledger_service,
    state::AppState,
    structs::{
        ledger::{
            CategoryList, LedgerEntry, LedgerListQuery, LedgerRequest, LedgerSummary, SummaryQuery,
        },
        members::AuthenticatedMember,
        pagination::Paginated,
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
    Router
};
use uuid::Uuid;

// 走 super::with_member_auth：寫入要進 admin_audit_logs（見 routes.rs 的說明）
pub fn new(state: AppState) -> Router<AppState> {
    super::with_member_auth(
        state,
        Router::new()
            .route("/", get(list).post(create))
            .route("/categories", get(categories))
            .route("/summary", get(summary))
            .route("/{id}", axum::routing::put(update).delete(delete)),
    )
}

async fn list(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<LedgerListQuery>,
) -> Result<Json<Paginated<LedgerEntry>>, AppError> {
    Ok(Json(
        ledger_service::list(state.get_pool(), auth_member.member_id, &query).await?,
    ))
}

async fn create(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<LedgerRequest>,
) -> Result<(StatusCode, Json<LedgerEntry>), AppError> {
    let entry = ledger_service::create(state.get_pool(), auth_member.member_id, &req).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn update(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<LedgerRequest>,
) -> Result<Json<LedgerEntry>, AppError> {
    Ok(Json(
        ledger_service::update(state.get_pool(), id, auth_member.member_id, &req).await?,
    ))
}

async fn delete(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    ledger_service::delete(state.get_pool(), id, auth_member.member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn summary(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<SummaryQuery>,
) -> Result<Json<LedgerSummary>, AppError> {
    Ok(Json(
        ledger_service::summary(state.get_pool(), auth_member.member_id, &query).await?,
    ))
}

async fn categories(
    Extension(_auth_member): Extension<AuthenticatedMember>,
) -> Json<CategoryList> {
    Json(ledger_service::categories())
}
