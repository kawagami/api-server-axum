use crate::{
    errors::AppError,
    repositories::logs::{get_logs, Log},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
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
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(get_logs_handler)))
}

async fn get_logs_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<Vec<Log>>, AppError> {
    auth_user.require_permission(Perm::LogRead)?;
    let logs = get_logs(state.get_pool(), query.level, query.limit, query.offset).await?;
    Ok(Json(logs))
}
