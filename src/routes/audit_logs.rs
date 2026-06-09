use crate::{
    errors::AppError,
    services::audit_logs::{get_audit_logs, AuditLog},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
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
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(get_audit_logs_handler)))
}

async fn get_audit_logs_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<Vec<AuditLog>>, AppError> {
    auth_user.require_permission(Perm::AuditRead)?;
    let logs = get_audit_logs(
        state.get_pool(),
        query.user_email,
        query.method,
        query.path,
        query.from,
        query.to,
        query.limit,
        query.offset,
    )
    .await?;
    Ok(Json(logs))
}
