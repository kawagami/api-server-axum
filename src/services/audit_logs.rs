use crate::{errors::AppError, repositories::audit_logs, state::AppState};
use chrono::{DateTime, Utc};

pub use crate::repositories::audit_logs::AuditLog;

pub async fn get_audit_logs(
    state: &AppState,
    user_email: Option<String>,
    method: Option<String>,
    path: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLog>, AppError> {
    audit_logs::get_audit_logs(
        state.get_pool(),
        user_email,
        method,
        path,
        from,
        to,
        limit,
        offset,
    )
    .await
    .map_err(AppError::from)
}
