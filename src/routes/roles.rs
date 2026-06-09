use crate::{
    errors::AppError,
    services::roles as roles_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::{NewRole, Perm, Role, RoleWithPermissions, SetRolePermissions},
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_roles).post(create_role))
            .route("/{id}", get(get_role).delete(delete_role))
            .route("/{id}/permissions", put(set_permissions)),
    )
}

async fn list_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Role>>, AppError> {
    auth_user.require_permission(Perm::RoleRead)?;
    Ok(Json(roles_service::get_roles(state.get_pool()).await?))
}

async fn get_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<RoleWithPermissions>, AppError> {
    auth_user.require_permission(Perm::RoleRead)?;
    Ok(Json(roles_service::get_role(state.get_pool(), id).await?))
}

async fn create_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(body): Json<NewRole>,
) -> Result<Json<Role>, AppError> {
    auth_user.require_permission(Perm::RoleCreate)?;
    Ok(Json(roles_service::create_role(state.get_pool(), body).await?))
}

async fn set_permissions(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<SetRolePermissions>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::RoleUpdate)?;
    roles_service::set_role_permissions(state.get_pool(), state.get_redis_pool(), id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::RoleDelete)?;
    roles_service::delete_role(state.get_pool(), state.get_redis_pool(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
