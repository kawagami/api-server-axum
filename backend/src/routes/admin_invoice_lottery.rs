use crate::{
    errors::AppError,
    services::invoices as invoices_service,
    state::AppState,
    structs::{auth::AuthenticatedUser, invoices::AdminLotteryNumbersRequest, roles::Perm},
};
use axum::{extract::{Extension, State}, routing::post, Json, Router};
use serde_json::json;

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", post(set_numbers)))
}

/// 手動補某期中獎號碼（後備來源），並觸發該期重新對獎
async fn set_numbers(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<AdminLotteryNumbersRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::InvoiceLotteryWrite)?;
    let stored = invoices_service::admin_set_numbers(state.get_pool(), &req).await?;
    Ok(Json(json!({ "period": req.period, "stored": stored })))
}
