use crate::state::AppState;
use axum::Router;

use super::{audit_logs, auth, permissions, roles, users};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::new(state.clone()))
        .nest("/users", users::new(state.clone()))
        .nest("/roles", roles::new(state.clone()))
        .nest("/permissions", permissions::new(state.clone()))
        .nest("/audit_logs", audit_logs::new(state))
}
