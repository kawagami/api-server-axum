use crate::extract::{Json, Query};
use crate::{
    errors::AppError,
    services::audit_logs::get_audit_logs,
    state::AppState,
    structs::{
        audit_logs::{AuditLog, AuditLogQuery},
        auth::AuthenticatedUser,
        pagination::{PageQuery, Paginated},
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, State},
    routing::get,
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(list_audit_logs)))
}

async fn list_audit_logs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Paginated<AuditLog>>, AppError> {
    auth_user.require_permission(Perm::AuditRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    Ok(Json(
        get_audit_logs(state.get_pool(), &query, limit, offset).await?,
    ))
}
