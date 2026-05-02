use crate::{
    errors::AppError,
    middleware::auth,
    services::roles as roles_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::{NewRole, Role, RoleWithPermissions, SetRolePermissions},
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    routing::{get, put},
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_roles).post(create_role))
        .route("/{id}", get(get_role).delete(delete_role))
        .route("/{id}/permissions", put(set_permissions))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ))
}

async fn list_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Role>>, AppError> {
    auth_user.require_permission("role:read")?;
    Ok(Json(roles_service::get_roles(&state).await?))
}

async fn get_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<RoleWithPermissions>, AppError> {
    auth_user.require_permission("role:read")?;
    Ok(Json(roles_service::get_role(&state, id).await?))
}

async fn create_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(body): Json<NewRole>,
) -> Result<Json<Role>, AppError> {
    auth_user.require_permission("role:create")?;
    Ok(Json(roles_service::create_role(&state, body).await?))
}

async fn set_permissions(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<SetRolePermissions>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission("role:update")?;
    roles_service::set_role_permissions(&state, id, body).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_role(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission("role:delete")?;
    roles_service::delete_role(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
