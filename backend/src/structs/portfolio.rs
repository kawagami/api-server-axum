use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PortfolioSummaryEntry {
    #[serde(flatten)]
    pub base: PortfolioEntry,
    pub stock_name: Option<String>,
    pub current_price: Option<f64>,
    pub current_value: Option<f64>,
    pub pnl: Option<f64>,
    pub pnl_pct: Option<f64>,
    /// 各期間的增減（今日 / 近一週 / 近一月）。資料不足的期間為 `None`。
    pub changes: PeriodChanges,
}

/// 一筆持股在某個期間的增減。基準價都已還原期間內的除權息 —— 少了這步，
/// 除息日會被算成一次大跌，而那正是這幾個數字最容易騙人的地方。
#[derive(Serialize)]
pub struct PeriodChange {
    /// 實際拿來比較的交易日（不一定等於目標日，市場休市就往前找最近的一天）
    pub base_date: NaiveDate,
    /// 基準日收盤價，已還原到今天的除權息基準
    pub base_close: f64,
    /// 每股增減
    pub change: f64,
    pub change_pct: f64,
    /// 這筆持股的市值增減（`change * shares`）
    pub value_change: f64,
}

/// 三個期間各一份。`None` = 那個期間沒有可用的基準日（新持股、或行情還沒補齊）。
#[derive(Serialize, Default)]
pub struct PeriodChanges {
    pub day: Option<PeriodChange>,
    pub week: Option<PeriodChange>,
    pub month: Option<PeriodChange>,
}

#[derive(Clone, Serialize, FromRow)]
pub struct PortfolioEntry {
    pub id: Uuid,
    pub member_id: i64,
    pub stock_code: String,
    pub buy_date: NaiveDate,
    pub cost_per_share: f64,
    pub shares: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `buy_date` 的下限年份。TWSE 個股日成交資訊（STOCK_DAY）最早只回到民國 81 年，
/// 比這更早的月份抓不到任何東西，只是讓逐月迴圈多跑幾輪。
pub const MIN_BUY_YEAR: i32 = 1992;

pub fn min_buy_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(MIN_BUY_YEAR, 1, 1).expect("1992-01-01 為合法日期")
}

#[derive(Deserialize)]
pub struct PortfolioRequest {
    pub stock_code: String,
    pub buy_date: NaiveDate,
    pub cost_per_share: f64,
    pub shares: i64,
}

impl PortfolioRequest {
    /// `buy_date` 是 `/summary` 與 `/{id}/history` 逐月迴圈的起點，而那個迴圈
    /// **每個月都會打一次 Redis 加一次 DB**（`UpstreamBudget` 只管 TWSE，不管這兩者），
    /// 查無資料時也不寫快取，所以每次請求都重跑一遍。沒有下限的話，一筆
    /// `buy_date=0001-01-01` 的持股就是兩萬多次序列查詢，而 `/summary` 還會對所有
    /// 持股平行跑 —— 單一登入會員即可吃光那 20 條連線的 pool。
    ///
    /// `today` 由呼叫端傳入（`utils::date::taipei_today()`），讓這支保持純函式可測。
    pub fn validate(&self, today: NaiveDate) -> Result<(), String> {
        if self.buy_date < min_buy_date() {
            return Err(format!("buy_date 不可早於 {}", min_buy_date()));
        }
        if self.buy_date > today {
            return Err("buy_date 不可晚於今日".to_string());
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct HistoryRecord {
    pub date: NaiveDate,
    pub close: f64,
    pub adjusted_cost: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(buy_date: &str) -> PortfolioRequest {
        PortfolioRequest {
            stock_code: "2330".to_string(),
            buy_date: buy_date.parse().expect("測試日期"),
            cost_per_share: 500.0,
            shares: 1000,
        }
    }

    fn today() -> NaiveDate {
        "2026-08-08".parse().expect("測試日期")
    }

    #[test]
    fn normal_buy_date_is_accepted() {
        assert!(req("2020-03-16").validate(today()).is_ok());
        // 邊界兩端都要收
        assert!(req("1992-01-01").validate(today()).is_ok());
        assert!(req("2026-08-08").validate(today()).is_ok());
    }

    #[test]
    fn buy_date_before_twse_data_is_rejected() {
        // 這一筆沒擋下來就是兩萬多次序列 Redis + DB 查詢
        assert!(req("0001-01-01").validate(today()).is_err());
        assert!(req("1991-12-31").validate(today()).is_err());
    }

    #[test]
    fn future_buy_date_is_rejected() {
        assert!(req("2026-08-09").validate(today()).is_err());
    }
}
