use crate::{
    errors::AppError,
    repositories::visitors::{self, DailyVisitorStat},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/visitors", get(visitors_stats)))
}

#[derive(Deserialize)]
struct VisitorsQuery {
    /// 歷史回看天數，預設 30，上限 365
    days: Option<i64>,
}

#[derive(Serialize)]
struct VisitorsStats {
    /// 今日（台北）即時不重複到訪，直接讀 Redis HLL
    today: DailyVisitorStat,
    /// 近 days 天合併去重（跨日不重複），讀 Redis HLL
    last_n_days_unique: i64,
    /// 已落地的歷史每日數字，新到舊
    history: Vec<DailyVisitorStat>,
}

/// 網站每日不重複到訪統計：今日即時值 + 期間合併去重 + 歷史趨勢。
async fn visitors_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<VisitorsQuery>,
) -> Result<Json<VisitorsStats>, AppError> {
    auth_user.require_permission(Perm::StatRead)?;

    let days = query.days.unwrap_or(30).clamp(1, 365);
    let today = visitors::taipei_today();

    let today_count = visitors::count_day(state.get_redis_pool(), today).await?;

    let range: Vec<NaiveDate> = (0..days).map(|i| today - Duration::days(i)).collect();
    let last_n_days_unique = visitors::count_days(state.get_redis_pool(), &range).await?;

    let history = visitors::history(state.get_pool(), days).await?;

    Ok(Json(VisitorsStats {
        today: DailyVisitorStat {
            date: today,
            unique_visitors: today_count,
        },
        last_n_days_unique,
        history,
    }))
}
