use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 某一天的不重複到訪數。DB（`daily_visitor_stats`）與即時 Redis HLL 共用同一個形狀。
#[derive(Serialize, sqlx::FromRow)]
pub struct DailyVisitorStat {
    pub date: NaiveDate,
    pub unique_visitors: i64,
}

/// `GET /admin/stats/visitors` 的查詢參數
#[derive(Deserialize)]
pub struct VisitorsQuery {
    /// 歷史回看天數，預設 30，上限 365
    pub days: Option<i64>,
}

/// `GET /admin/stats/visitors` 的回應
#[derive(Serialize)]
pub struct VisitorsStats {
    /// 今日（台北）即時不重複到訪，直接讀 Redis HLL
    pub today: DailyVisitorStat,
    /// 近 days 天合併去重（跨日不重複），讀 Redis HLL
    pub last_n_days_unique: i64,
    /// 已落地的歷史每日數字，新到舊
    pub history: Vec<DailyVisitorStat>,
}
