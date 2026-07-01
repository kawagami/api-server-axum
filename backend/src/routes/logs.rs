use crate::{
    errors::AppError,
    repositories::logs::Log,
    services::logs as logs_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, pagination::PageQuery, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct LogQuery {
    level: Option<String>,
}

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(get_logs_handler)))
}

async fn get_logs_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Vec<Log>>, AppError> {
    auth_user.require_permission(Perm::LogRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    let logs = logs_service::get_logs(state.get_pool(), query.level, limit, offset).await?;
    Ok(Json(logs))
}
