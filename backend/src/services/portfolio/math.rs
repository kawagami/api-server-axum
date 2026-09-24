use crate::structs::portfolio::{HistoryRecord, PeriodChange, PeriodChanges};
use chrono::{Days, Months, NaiveDate};

pub(super) struct DayClose {
    pub(super) date: NaiveDate,
    pub(super) close: f64,
}

pub(super) struct ExEvent {
    pub(super) date: NaiveDate,
    pub(super) close_before: f64,
    pub(super) cash_div: f64,
    pub(super) stock_rate: f64,
}

/// `compute_latest` 的結果。欄位一多就不該再用 tuple —— summary 現在要的是
/// 「相對成本的損益」加「相對三個期間起點的增減」兩組數字。
pub(super) struct LatestSnapshot {
    pub(super) current_price: f64,
    pub(super) current_value: f64,
    pub(super) pnl: f64,
    pub(super) pnl_pct: f64,
    pub(super) changes: PeriodChanges,
}

/// 基準日最多可以比目標日早幾天。
///
/// **上下限都有理由**：TWSE 春節連假可以連休 9 天，設太緊會讓農曆年前後的「近一週」
/// 整段變成 `None`；但也不能不設 —— `UpstreamBudget` 逾預算的月份是**整段沒有資料**，
/// 那時「最接近且不晚於目標日的一筆」可能是三個月前，拿它算出來的東西不叫「近一週」，
/// 而使用者從畫面上看不出差別。寧可顯示「-」也不要顯示一個名不副實的數字。
const MAX_BASE_LOOKBACK_DAYS: i64 = 10;

/// 除權息還原因子：把「除權息前」的價格換算成「除權息後」的可比價格。
/// 成本調整與前收盤價調整用的是同一個因子，所以抽出來共用。
fn ex_adjust_factor(ev: &ExEvent) -> Option<f64> {
    if ev.close_before <= 0.0 {
        return None;
    }
    let numer = ev.close_before - ev.cash_div;
    let denom = ev.close_before * (1.0 + ev.stock_rate / 1000.0);
    (denom > 0.0).then_some(numer / denom)
}

pub(super) fn compute_latest(
    cost: f64,
    shares: i64,
    closes: &[DayClose],
    mut ex_events: Vec<ExEvent>,
) -> Option<LatestSnapshot> {
    let last = closes.last()?;
    ex_events.sort_by_key(|e| e.date);

    let mut adjusted_cost = cost;
    for ev in &ex_events {
        if ev.date > last.date {
            break;
        }
        if let Some(f) = ex_adjust_factor(ev) {
            adjusted_cost *= f;
        }
    }

    let pnl = (last.close - adjusted_cost) * shares as f64;
    let pnl_pct = if adjusted_cost != 0.0 {
        (last.close - adjusted_cost) / adjusted_cost * 100.0
    } else {
        0.0
    };

    // 三個期間的目標日都用日曆算（使用者說的「近一週」是七天前，不是七個交易日前），
    // 再由 `period_change` 往前找最近的交易日。
    let changes = PeriodChanges {
        day: last
            .date
            .checked_sub_days(Days::new(1))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
        week: last
            .date
            .checked_sub_days(Days::new(7))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
        month: last
            .date
            .checked_sub_months(Months::new(1))
            .and_then(|t| period_change(shares, closes, &ex_events, last, t)),
    };

    Some(LatestSnapshot {
        current_price: last.close,
        current_value: last.close * shares as f64,
        pnl,
        pnl_pct,
        changes,
    })
}

/// 以 `target` 當目標日算一個期間的增減。`closes` 需已按日期遞增排序、`ex_events` 已排序。
///
/// 基準日 = **最後一個不晚於 `target` 的交易日**（市場休市就自然往前落），
/// 太舊則放棄（見 `MAX_BASE_LOOKBACK_DAYS`）。
fn period_change(
    shares: i64,
    closes: &[DayClose],
    ex_events: &[ExEvent],
    last: &DayClose,
    target: NaiveDate,
) -> Option<PeriodChange> {
    let idx = closes.partition_point(|d| d.date <= target).checked_sub(1)?;
    let base = &closes[idx];
    // 同一天沒有增減可言（`target` 落在最後一個交易日當天或之後）
    if base.date >= last.date {
        return None;
    }
    if (target - base.date).num_days() > MAX_BASE_LOOKBACK_DAYS {
        return None;
    }

    // 兩日之間的除權息要還原到基準價上，否則除息日會被當成一次大跌
    let mut base_close = base.close;
    for ev in ex_events {
        if ev.date > base.date && ev.date <= last.date {
            if let Some(f) = ex_adjust_factor(ev) {
                base_close *= f;
            }
        }
    }

    let change = last.close - base_close;
    let change_pct = if base_close != 0.0 {
        change / base_close * 100.0
    } else {
        0.0
    };

    Some(PeriodChange {
        base_date: base.date,
        base_close,
        change,
        change_pct,
        value_change: change * shares as f64,
    })
}

pub(super) fn build_history(
    cost: f64,
    shares: i64,
    closes: Vec<DayClose>,
    mut ex_events: Vec<ExEvent>,
) -> Vec<HistoryRecord> {
    ex_events.sort_by_key(|e| e.date);

    let mut adjusted_cost = cost;
    let mut applied = 0usize;
    let mut records = Vec::with_capacity(closes.len());

    for day in &closes {
        while applied < ex_events.len() && ex_events[applied].date <= day.date {
            if let Some(f) = ex_adjust_factor(&ex_events[applied]) {
                adjusted_cost *= f;
            }
            applied += 1;
        }

        let pnl = (day.close - adjusted_cost) * shares as f64;
        let pnl_pct = if adjusted_cost != 0.0 {
            (day.close - adjusted_cost) / adjusted_cost * 100.0
        } else {
            0.0
        };

        records.push(HistoryRecord {
            date: day.date,
            close: day.close,
            adjusted_cost,
            pnl,
            pnl_pct,
        });
    }

    records
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn d(s: &str) -> NaiveDate {
        s.parse().expect("測試日期")
    }

    fn closes(rows: &[(&str, f64)]) -> Vec<DayClose> {
        rows.iter().map(|(dt, c)| DayClose { date: d(dt), close: *c }).collect()
    }

    /// 從 `from` 起連續 n 個交易日（跳週末），收盤價逐日 +1
    fn daily_series(from: &str, n: i64, start_close: f64) -> Vec<DayClose> {
        let mut out = Vec::new();
        let mut date = d(from);
        let mut close = start_close;
        while (out.len() as i64) < n {
            if date.weekday().num_days_from_monday() < 5 {
                out.push(DayClose { date, close });
                close += 1.0;
            }
            date = date.succ_opt().expect("測試日期");
        }
        out
    }

    #[test]
    fn day_change_is_relative_to_previous_trading_day() {
        // 週五 → 下週一：目標日（週日）沒開盤，基準自然落回週五
        let c = closes(&[("2026-09-04", 100.0), ("2026-09-07", 110.0)]);
        let day = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes.day.expect("有前一交易日");

        assert_eq!(day.base_date, d("2026-09-04"));
        assert_eq!(day.base_close, 100.0);
        assert_eq!(day.change, 10.0);
        assert_eq!(day.change_pct, 10.0);
        assert_eq!(day.value_change, 10_000.0);
    }

    #[test]
    fn single_day_has_no_change_in_any_period() {
        let c = closes(&[("2026-09-07", 110.0)]);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        assert!(ch.day.is_none());
        assert!(ch.week.is_none());
        assert!(ch.month.is_none());
    }

    #[test]
    fn ex_dividend_day_is_not_reported_as_a_crash() {
        // 前一日收 100、配息 5 元，除息日開平收 95：帳面是 -5，實際沒漲沒跌。
        // 少了基準價的還原，這天會顯示 -5%（增減數字最容易騙人的地方）。
        let c = closes(&[("2026-09-04", 100.0), ("2026-09-07", 95.0)]);
        let ev = vec![ExEvent { date: d("2026-09-07"), close_before: 100.0, cash_div: 5.0, stock_rate: 0.0 }];
        let day = compute_latest(80.0, 1000, &c, ev).expect("有收盤價").changes.day.expect("有前一交易日");

        assert_eq!(day.base_close, 95.0);
        assert_eq!(day.change, 0.0);
        assert_eq!(day.value_change, 0.0);
    }

    #[test]
    fn week_and_month_pick_the_nearest_trading_day_at_or_before_target() {
        // 2026-08-03(一) 起 30 個交易日，收盤 100,101,…；最後一天是 2026-09-11(五) 收 129
        let c = daily_series("2026-08-03", 30, 100.0);
        let last = c.last().expect("有資料");
        assert_eq!(last.date, d("2026-09-11"));

        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        // 一週前 = 09-04(五)，當天有開盤
        let w = ch.week.expect("有一週前");
        assert_eq!(w.base_date, d("2026-09-04"));
        assert_eq!(w.change, 5.0);
        assert_eq!(w.value_change, 5_000.0);

        // 一個月前 = 08-11(二)，當天有開盤
        let m = ch.month.expect("有一個月前");
        assert_eq!(m.base_date, d("2026-08-11"));
        assert_eq!(m.change, 23.0);
    }

    #[test]
    fn period_is_none_when_position_is_too_new() {
        // 只有三個交易日：有今日、沒有一週前與一個月前
        let c = daily_series("2026-09-09", 3, 100.0);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價").changes;

        assert!(ch.day.is_some());
        assert!(ch.week.is_none());
        assert!(ch.month.is_none());
    }

    #[test]
    fn stale_baseline_is_rejected_rather_than_mislabelled() {
        // 行情有大洞（UpstreamBudget 逾預算的月份會整段沒資料）：目標日往前 10 天內
        // 找不到交易日就回 None —— 顯示「-」勝過拿三個月前的價格謊稱是「近一週」。
        let c = closes(&[("2026-06-01", 100.0), ("2026-09-07", 130.0)]);
        let ch = compute_latest(80.0, 1000, &c, vec![]).expect("有收盤價");

        assert!(ch.changes.day.is_none());
        assert!(ch.changes.week.is_none());
        assert!(ch.changes.month.is_none());
    }
}
