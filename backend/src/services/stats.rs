use crate::{
    errors::AppError,
    repositories::visitors,
    state::AppState,
    structs::stats::{DailyVisitorStat, VisitorsStats},
};
use chrono::{Duration, NaiveDate};

/// 歷史回看天數上限 —— `count_days` 是一次 `PFCOUNT` 多把 key，天數即 key 數。
const MAX_DAYS: i64 = 365;
const DEFAULT_DAYS: i64 = 30;

/// 網站每日不重複到訪：今日即時值 + 期間合併去重 + 歷史趨勢。
///
/// 三支查詢（Redis 兩支、DB 一支）彼此無依賴，併發跑。原本是三支序列 `await`，
/// 而且整段邏輯待在 route 層 —— 那是這支端點唯一直接呼叫 repository 的原因。
///
/// `repositories::visitors` 的回傳錯誤型別是 `RedisError` / `sqlx::Error`（該檔尚未
/// 統一成 `AppError`），所以每支都先轉型才進得了同一個 `try_join!`。
pub async fn visitors_stats(state: &AppState, days: Option<i64>) -> Result<VisitorsStats, AppError> {
    let days = days.unwrap_or(DEFAULT_DAYS).clamp(1, MAX_DAYS);
    let today = crate::utils::date::taipei_today();
    let range: Vec<NaiveDate> = (0..days).map(|i| today - Duration::days(i)).collect();

    let today_count = async {
        visitors::count_day(state.get_redis_pool(), today)
            .await
            .map_err(AppError::from)
    };
    let last_n_days_unique = async {
        visitors::count_days(state.get_redis_pool(), &range)
            .await
            .map_err(AppError::from)
    };
    let history = async {
        visitors::history(state.get_pool(), days)
            .await
            .map_err(AppError::from)
    };

    let (today_count, last_n_days_unique, history) =
        tokio::try_join!(today_count, last_n_days_unique, history)?;

    Ok(VisitorsStats {
        today: DailyVisitorStat { date: today, unique_visitors: today_count },
        last_n_days_unique,
        history,
    })
}

/// WS 握手成功時記一次到訪（best-effort，失敗只 warn，不阻塞連線）。
///
/// 採集點在 WS 而不是 HTTP：前台頁載入即連 WS，天然濾掉不跑 JS 的 bot/爬蟲。
pub async fn record_visit(state: &AppState, ip: &str, user_agent: &str) {
    visitors::record_visit(state.get_redis_pool(), ip, user_agent).await;
}
