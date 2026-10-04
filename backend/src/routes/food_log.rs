use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::food_log as food_log_service,
    state::AppState,
    structs::{
        food_log::{
            FoodLogDay, FoodLogDayNoteRequest, FoodLogDaysQuery, FoodLogEntry, FoodLogEntryRequest,
            FoodLogSuggestion, FoodLogSummary,
        },
        members::AuthenticatedMember,
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, post, put},
    Router
};
use chrono::NaiveDate;

// 走 super::with_member_auth：寫入要進 admin_audit_logs（audit 只記 path / query，不記 body）
pub fn new(state: AppState) -> Router<AppState> {
    super::with_member_auth(
        state,
        Router::new()
            .route("/days", get(list_days))
            .route("/days/{date}", put(set_day_note))
            .route("/entries", post(create))
            .route("/entries/{id}", put(update).delete(delete))
            .route("/suggestions", get(suggestions))
            .route("/summary", get(summary)),
    )
}

async fn list_days(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Query(query): Query<FoodLogDaysQuery>,
) -> Result<Json<Vec<FoodLogDay>>, AppError> {
    Ok(Json(food_log_service::list_days(state.get_pool(), auth_member.member_id, &query).await?))
}

async fn set_day_note(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(date): Path<NaiveDate>,
    Json(req): Json<FoodLogDayNoteRequest>,
) -> Result<StatusCode, AppError> {
    food_log_service::set_day_note(state.get_pool(), auth_member.member_id, date, req).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Json(req): Json<FoodLogEntryRequest>,
) -> Result<(StatusCode, Json<FoodLogEntry>), AppError> {
    let entry = food_log_service::create(state.get_pool(), auth_member.member_id, req).await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

async fn update(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<FoodLogEntryRequest>,
) -> Result<Json<FoodLogEntry>, AppError> {
    Ok(Json(food_log_service::update(state.get_pool(), id, auth_member.member_id, req).await?))
}

async fn delete(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    food_log_service::delete(state.get_pool(), id, auth_member.member_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn suggestions(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<Vec<FoodLogSuggestion>>, AppError> {
    Ok(Json(food_log_service::suggestions(state.get_pool(), auth_member.member_id).await?))
}

async fn summary(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<FoodLogSummary>, AppError> {
    Ok(Json(food_log_service::summary(state.get_pool(), auth_member.member_id).await?))
}
