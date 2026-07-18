use crate::{
    errors::AppError,
    services::gov_tenders as gov_tenders_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        gov_tenders::{GovTenderListQuery, GovTenderPaginatedResponse},
        pagination::PageQuery,
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_tenders))
            .route("/types", get(list_types)),
    )
}

/// 標案公告分頁列表（?keyword=&tender_type=&q=&page=&per_page=）
async fn list_tenders(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<GovTenderListQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<GovTenderPaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::GovTenderRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        gov_tenders_service::list(state.get_pool(), &filter, limit, offset).await?,
    ))
}

/// 所有出現過的標案類型（去重、排序，供篩選下拉）
async fn list_types(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, AppError> {
    auth_user.require_permission(Perm::GovTenderRead)?;
    Ok(Json(gov_tenders_service::types(state.get_pool()).await?))
}
