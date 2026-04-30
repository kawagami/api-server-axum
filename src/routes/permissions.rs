use crate::{
    errors::AppError,
    middleware::auth,
    services::roles as roles_service,
    state::AppStateV2,
    structs::{auth::AuthenticatedUser, roles::Permission},
};
use axum::{
    extract::{Extension, State},
    middleware,
    routing::get,
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/", get(list_permissions))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ))
}

async fn list_permissions(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<Permission>>, AppError> {
    auth_user.require_permission("role:read")?;
    Ok(Json(roles_service::get_permissions(&state).await?))
}
