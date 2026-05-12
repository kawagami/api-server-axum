use crate::{
    errors::AppError,
    middleware::auth,
    repositories::audit_logs::{get_audit_logs, AuditLog},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    middleware,
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
    Router::new()
        .route("/", get(get_audit_logs_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ))
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
