use crate::{
    errors::AppError,
    services::audit_logs::{get_audit_logs, AuditLog},
    state::AppState,
    structs::{auth::AuthenticatedUser, pagination::PageQuery, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct AuditLogQuery {
    user_email: Option<String>,
    method: Option<String>,
    path: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(get_audit_logs_handler)))
}

async fn get_audit_logs_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Vec<AuditLog>>, AppError> {
    auth_user.require_permission(Perm::AuditRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    let logs = get_audit_logs(
        state.get_pool(),
        query.user_email,
        query.method,
        query.path,
        query.from,
        query.to,
        limit,
        offset,
    )
    .await?;
    Ok(Json(logs))
}
