use crate::{
    errors::AppError,
    middleware::auth,
    services::members as members_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        members::{Member, MemberDetail},
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, Path, State},
    middleware,
    routing::get,
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_members))
        .route("/{id}", get(get_member_by_id))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ))
}

async fn get_members(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Member>>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    Ok(Json(members_service::get_members(&state).await?))
}

async fn get_member_by_id(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Option<MemberDetail>>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    Ok(Json(members_service::get_member_by_id(&state, id).await?))
}
