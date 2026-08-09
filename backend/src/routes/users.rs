use crate::extract::{Json, Path};
use crate::{
    errors::{AppError, RequestError},
    services::users as users_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::{Perm, Role, SetUserRoles},
        users::{NewUser, User},
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route(
                "/",
                get(list_users).post(create_user).delete(delete_user),
            )
            .route("/{id}/roles", get(user_roles).put(set_user_roles)),
    )
}

async fn list_users(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    auth_user.require_permission(Perm::UserRead)?;
    Ok(Json(users_service::get_users(state.get_pool()).await?))
}

async fn create_user(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(user): Json<NewUser>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::UserCreate)?;
    // 帶 role_ids 就等於在指派角色，門檻必須與 PUT /admin/users/{id}/roles 一致。
    // 少了這道，只有 user:create 的管理員可以繞過整個 role:assign 的把關。
    if !user.role_ids.is_empty() {
        auth_user.require_permission(Perm::RoleAssign)?;
    }
    users_service::create_user(state.get_pool(), &state.get_settings(), &auth_user, user).await?;
    Ok(StatusCode::CREATED)
}

async fn delete_user(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::UserDelete)?;
    users_service::delete_user(state.get_pool(), state.get_redis_pool(), user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn user_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<Role>>, AppError> {
    auth_user.require_permission(Perm::RoleRead)?;
    Ok(Json(users_service::get_user_roles(state.get_pool(), user_id).await?))
}

async fn set_user_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(body): Json<SetUserRoles>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::RoleAssign)?;
    // 不可改自己的角色：否則有 role:assign 的人可以自行加掛任何角色（自我提權）。
    // 要調整自己的權限得請另一位管理員操作。
    if user_id == auth_user.id {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            "不可變更自己的角色，請由其他管理員操作".to_string(),
        )));
    }
    users_service::set_user_roles(
        state.get_pool(),
        state.get_redis_pool(),
        &auth_user,
        user_id,
        body.role_ids,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
