use crate::{
    errors::AppError,
    middleware::auth,
    services::users as users_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::{Perm, Role, SetUserRoles},
        users::{NewUser, User},
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
    let protected_routes = Router::new()
        .route("/", axum::routing::post(create_user).delete(delete_user))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ));

    let role_routes = Router::new()
        .route("/{id}/roles", get(get_user_roles).put(set_user_roles))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ));

    Router::new()
        .route("/", get(get_users))
        .merge(protected_routes)
        .merge(role_routes)
}

async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    Ok(Json(users_service::get_users(&state).await?))
}

async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<NewUser>,
) -> Result<Json<bool>, AppError> {
    users_service::create_user(&state, user).await?;
    Ok(Json(true))
}

async fn delete_user(
    State(_state): State<AppState>,
    Json(_user): Json<User>,
) -> Result<Json<bool>, AppError> {
    Ok(Json(true))
}

async fn get_user_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<Role>>, AppError> {
    auth_user.require_permission(Perm::RoleRead)?;
    Ok(Json(users_service::get_user_roles(&state, user_id).await?))
}

async fn set_user_roles(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(body): Json<SetUserRoles>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::RoleAssign)?;
    users_service::set_user_roles(&state, user_id, body.role_ids).await?;
    Ok(StatusCode::NO_CONTENT)
}
