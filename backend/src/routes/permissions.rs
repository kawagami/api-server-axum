use crate::{
    errors::AppError,
    services::roles as roles_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::{Perm, Permission}},
};
use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(list_permissions)))
}

async fn list_permissions(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Permission>>, AppError> {
    auth_user.require_permission(Perm::RoleRead)?;
    Ok(Json(roles_service::get_permissions(state.get_pool()).await?))
}
