use crate::{
    errors::AppError,
    middleware::auth,
    services::members as members_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        members::{AuthenticatedMember, Member, MemberDetail},
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
    // 走 super::with_auth 而不是直接掛 authorize_and_load：這兩支會吐會員個資（需
    // member:read），必須進 admin_audit_logs。直接掛 auth middleware 會跳過 audit 層。
    let admin_routes = super::with_auth(
        state.clone(),
        Router::new()
            .route("/", get(list_members))
            .route("/{id}", get(member_detail)),
    );

    let member_routes = Router::new()
        .route("/me", get(me))
        .layer(middleware::from_fn_with_state(state, auth::authorize_member));

    admin_routes.merge(member_routes)
}

async fn list_members(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Member>>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    Ok(Json(members_service::get_members(state.get_pool()).await?))
}

async fn member_detail(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Option<MemberDetail>>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    Ok(Json(members_service::get_member_by_id(state.get_pool(), id).await?))
}

async fn me(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<Option<MemberDetail>>, AppError> {
    Ok(Json(members_service::get_member_by_id(state.get_pool(), auth_member.member_id).await?))
}
