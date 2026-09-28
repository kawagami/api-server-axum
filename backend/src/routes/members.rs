use crate::extract::{Json, Path, Query};
use crate::{
    errors::{AppError, AuthError, RequestError},
    middleware::auth,
    services::members as members_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        members::{AuthenticatedMember, Member, MemberDetail},
        pagination::{PageQuery, Paginated},
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, State},
    middleware,
    routing::get,
    Router
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
    Query(page): Query<PageQuery>,
) -> Result<Json<Paginated<Member>>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    // 預設取 PageQuery 的上限（200）：前端目前沒有分頁 UI、預期拿完整清單，
    // 所以先只把查詢加界、不改行為。會員數逼近 200 時前端要補分頁。
    let (limit, offset) = page.to_limit_offset(crate::structs::pagination::MAX_PER_PAGE);
    Ok(Json(
        members_service::get_members(state.get_pool(), limit, offset).await?,
    ))
}

async fn member_detail(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<MemberDetail>, AppError> {
    auth_user.require_permission(Perm::MemberRead)?;
    // 查無此人回 404，不回 200 + null（前端照 MemberDetail 讀欄位會直接炸）
    members_service::get_member_by_id(state.get_pool(), id)
        .await?
        .map(Json)
        .ok_or(RequestError::NotFound.into())
}

async fn me(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<MemberDetail>, AppError> {
    // token 有效但帳號已不存在（被刪）→ 401：這張 token 已經不代表任何人，前端會導回登入頁
    members_service::get_member_by_id(state.get_pool(), auth_member.member_id)
        .await?
        .map(Json)
        .ok_or(AuthError::Unauthorized.into())
}
