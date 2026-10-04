use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::utils::text::normalize_optional;

/// 餐別。DB 存 `as_str()` 的字串（migration 有 CHECK 約束），宣告順序 = 一天之內的顯示順序。
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Meal {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
    LateNight,
    Other,
}

impl Meal {
    /// 顯示順序。排序在 SQL 端用 `array_position` 對這份清單做，順序只有這一份。
    pub const ALL: [Meal; 6] = [
        Meal::Breakfast,
        Meal::Lunch,
        Meal::Dinner,
        Meal::Snack,
        Meal::LateNight,
        Meal::Other,
    ];

    pub fn order() -> Vec<&'static str> {
        Meal::ALL.iter().map(|m| m.as_str()).collect()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Meal::Breakfast => "breakfast",
            Meal::Lunch => "lunch",
            Meal::Dinner => "dinner",
            Meal::Snack => "snack",
            Meal::LateNight => "late_night",
            Meal::Other => "other",
        }
    }
}

#[derive(Serialize, FromRow, Clone, Debug, PartialEq)]
pub struct FoodLogEntry {
    pub id: i64,
    pub eaten_on: NaiveDate,
    pub meal: String,
    pub meal_label: Option<String>,
    pub store: Option<String>,
    pub item: String,
    pub quantity: i16,
    /// 該行總價；`None` = 點數兌換 / 請客
    pub amount: Option<i32>,
    pub rating: Option<i16>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `eaten_on` 的下限。沒有業務意義，只是擋掉打錯世紀的日期（那種紀錄永遠翻不到）。
pub const MIN_EATEN_ON: (i32, u32, u32) = (2000, 1, 1);
pub const MAX_ITEM_LEN: usize = 100;
pub const MAX_STORE_LEN: usize = 100;
pub const MAX_MEAL_LABEL_LEN: usize = 50;
pub const MAX_NOTE_LEN: usize = 1000;
pub const MAX_DAY_NOTE_LEN: usize = 2000;
pub const MAX_QUANTITY: i16 = 99;
pub const MAX_AMOUNT: i32 = 1_000_000;

fn min_eaten_on() -> NaiveDate {
    let (y, m, d) = MIN_EATEN_ON;
    NaiveDate::from_ymd_opt(y, m, d).expect("MIN_EATEN_ON 為合法日期")
}

/// 新增 / 修改共用（PUT 是整筆覆寫）。
#[derive(Deserialize, Clone, Debug)]
pub struct FoodLogEntryRequest {
    pub eaten_on: NaiveDate,
    pub meal: Meal,
    pub meal_label: Option<String>,
    pub store: Option<String>,
    pub item: String,
    pub quantity: Option<i16>,
    pub amount: Option<i32>,
    pub rating: Option<i16>,
    pub note: Option<String>,
}

/// 驗證過、修剪過的一筆，repository 只吃這個。
#[derive(Debug, PartialEq)]
pub struct ValidFoodLogEntry {
    pub eaten_on: NaiveDate,
    pub meal: Meal,
    pub meal_label: Option<String>,
    pub store: Option<String>,
    pub item: String,
    pub quantity: i16,
    pub amount: Option<i32>,
    pub rating: Option<i16>,
    pub note: Option<String>,
}

fn check_len(field: &str, value: Option<&str>, max: usize) -> Result<(), String> {
    match value {
        Some(v) if v.chars().count() > max => Err(format!("{field} 不可超過 {max} 字")),
        _ => Ok(()),
    }
}

impl FoodLogEntryRequest {
    /// 空白字串收成 `None`、檢查範圍。`today` 由呼叫端傳入（`taipei_today()`），保持純函式可測。
    pub fn validate(self, today: NaiveDate) -> Result<ValidFoodLogEntry, String> {
        if self.eaten_on < min_eaten_on() {
            return Err(format!("eaten_on 不可早於 {}", min_eaten_on()));
        }
        if self.eaten_on > today {
            return Err("eaten_on 不可晚於今日".to_string());
        }

        let item = self.item.trim().to_string();
        if item.is_empty() {
            return Err("item 不可為空".to_string());
        }
        check_len("item", Some(&item), MAX_ITEM_LEN)?;

        let store = normalize_optional(self.store);
        let meal_label = normalize_optional(self.meal_label);
        let note = normalize_optional(self.note);
        check_len("store", store.as_deref(), MAX_STORE_LEN)?;
        check_len("meal_label", meal_label.as_deref(), MAX_MEAL_LABEL_LEN)?;
        check_len("note", note.as_deref(), MAX_NOTE_LEN)?;

        let quantity = self.quantity.unwrap_or(1);
        if !(1..=MAX_QUANTITY).contains(&quantity) {
            return Err(format!("quantity 必須介於 1 到 {MAX_QUANTITY}"));
        }
        if let Some(amount) = self.amount {
            if !(0..=MAX_AMOUNT).contains(&amount) {
                return Err(format!("amount 必須介於 0 到 {MAX_AMOUNT}"));
            }
        }
        if let Some(rating) = self.rating {
            if !(1..=5).contains(&rating) {
                return Err("rating 必須介於 1 到 5".to_string());
            }
        }

        Ok(ValidFoodLogEntry {
            eaten_on: self.eaten_on,
            meal: self.meal,
            meal_label,
            store,
            item,
            quantity,
            amount: self.amount,
            rating: self.rating,
            note,
        })
    }
}

/// `GET /days` 單次最多涵蓋的天數。前端一次翻一週，這只是擋掉一發撈全部歷史。
pub const MAX_DAYS_SPAN: i64 = 92;
/// 不給 `from` 時往回看幾天（含 `to` 當天）
pub const DEFAULT_DAYS_SPAN: i64 = 7;

#[derive(Deserialize)]
pub struct FoodLogDaysQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

impl FoodLogDaysQuery {
    /// 回 `(from, to)`，兩端皆含。`to` 預設今天、`from` 預設往回 `DEFAULT_DAYS_SPAN` 天。
    pub fn resolve(&self, today: NaiveDate) -> Result<(NaiveDate, NaiveDate), String> {
        let to = self.to.unwrap_or(today);
        let from = self.from.unwrap_or(to - Duration::days(DEFAULT_DAYS_SPAN - 1));
        if from > to {
            return Err("from 不可晚於 to".to_string());
        }
        if (to - from).num_days() >= MAX_DAYS_SPAN {
            return Err(format!("查詢區間不可超過 {MAX_DAYS_SPAN} 天"));
        }
        Ok((from, to))
    }
}

/// 時間軸的一天：當天的品項（已依餐別排序）、合計與當天備註。
#[derive(Serialize, Debug, PartialEq)]
pub struct FoodLogDay {
    pub date: NaiveDate,
    pub note: Option<String>,
    /// 當天有標金額的品項加總（點數兌換那些不算）
    pub total: i64,
    pub entries: Vec<FoodLogEntry>,
}

#[derive(FromRow)]
pub struct FoodLogDayNote {
    pub day: NaiveDate,
    pub note: String,
}

#[derive(Deserialize)]
pub struct FoodLogDayNoteRequest {
    /// 空白 = 刪掉當天備註
    pub note: String,
}

/// 各餐別吃過幾次。前端切餐別時用它重排「這餐常吃」，不必再打一次 API。
#[derive(Serialize, FromRow, Debug, Default)]
pub struct MealCounts {
    pub breakfast: i64,
    pub lunch: i64,
    pub dinner: i64,
    pub snack: i64,
    pub late_night: i64,
    pub other: i64,
}

/// 「常吃 / 最近」選項：同一個 (store, item) 一列，帶最近一次的數量與金額供一鍵帶入。
#[derive(Serialize, FromRow, Debug)]
pub struct FoodLogSuggestion {
    pub store: Option<String>,
    pub item: String,
    pub last_quantity: i16,
    pub last_amount: Option<i32>,
    pub last_eaten_on: NaiveDate,
    pub count: i64,
    #[sqlx(flatten)]
    pub meal_counts: MealCounts,
}

/// 選項只看最近這麼多天：太久沒吃的東西不該擠掉常吃的。
pub const SUGGESTION_LOOKBACK_DAYS: i64 = 365;
pub const SUGGESTION_LIMIT: i64 = 300;

/// 今日 / 本週（週一起）/ 本月花費，台北日界。
#[derive(Serialize, FromRow, Debug, PartialEq)]
pub struct FoodLogSummary {
    pub today: i64,
    pub week: i64,
    pub month: i64,
}

/// 本週一
pub fn week_start(today: NaiveDate) -> NaiveDate {
    today - Duration::days(i64::from(today.weekday().num_days_from_monday()))
}

/// 本月 1 日
pub fn month_start(today: NaiveDate) -> NaiveDate {
    today.with_day(1).expect("每月必有 1 日")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> NaiveDate {
        s.parse().expect("測試日期")
    }

    fn req() -> FoodLogEntryRequest {
        FoodLogEntryRequest {
            eaten_on: date("2026-10-04"),
            meal: Meal::Lunch,
            meal_label: None,
            store: Some("小眷村牛肉麵".into()),
            item: "大老鼠麵".into(),
            quantity: None,
            amount: Some(120),
            rating: None,
            note: None,
        }
    }

    const TODAY: &str = "2026-10-04";

    #[test]
    fn minimal_entry_defaults_quantity_to_one() {
        let v = req().validate(date(TODAY)).unwrap();
        assert_eq!(v.quantity, 1);
        assert_eq!(v.item, "大老鼠麵");
    }

    /// 選填欄位送空白字串 = 沒填，DB 只該有 NULL 一種「沒填」
    #[test]
    fn blank_optionals_become_none_and_item_is_trimmed() {
        let mut r = req();
        r.store = Some("   ".into());
        r.note = Some("".into());
        r.meal_label = Some(" \t".into());
        r.item = "  牛丼 ".into();
        let v = r.validate(date(TODAY)).unwrap();
        assert_eq!((v.store, v.note, v.meal_label), (None, None, None));
        assert_eq!(v.item, "牛丼");
    }

    #[test]
    fn blank_item_is_rejected() {
        let mut r = req();
        r.item = "   ".into();
        assert!(r.validate(date(TODAY)).is_err());
    }

    #[test]
    fn date_bounds() {
        let mut r = req();
        r.eaten_on = date("2026-10-05");
        assert!(r.clone().validate(date(TODAY)).is_err(), "未來日期");
        r.eaten_on = date("1999-12-31");
        assert!(r.clone().validate(date(TODAY)).is_err(), "早於下限");
        r.eaten_on = date("2000-01-01");
        assert!(r.validate(date(TODAY)).is_ok(), "下限本身要收");
    }

    #[test]
    fn numeric_bounds() {
        let ok = |f: fn(&mut FoodLogEntryRequest)| {
            let mut r = req();
            f(&mut r);
            r.validate(date(TODAY)).is_ok()
        };
        assert!(ok(|r| r.amount = None), "點數兌換：沒有金額");
        assert!(ok(|r| r.amount = Some(0)));
        assert!(!ok(|r| r.amount = Some(-1)));
        assert!(!ok(|r| r.amount = Some(MAX_AMOUNT + 1)));
        assert!(!ok(|r| r.quantity = Some(0)));
        assert!(ok(|r| r.quantity = Some(MAX_QUANTITY)));
        assert!(ok(|r| r.rating = Some(5)));
        assert!(!ok(|r| r.rating = Some(0)));
        assert!(!ok(|r| r.rating = Some(6)));
    }

    /// 長度以字元計，不是位元組 —— 中文一字 3 bytes，用 len() 會把上限砍成三分之一
    #[test]
    fn length_counts_chars_not_bytes() {
        let mut r = req();
        r.item = "麵".repeat(MAX_ITEM_LEN);
        assert!(r.clone().validate(date(TODAY)).is_ok());
        r.item = "麵".repeat(MAX_ITEM_LEN + 1);
        assert!(r.validate(date(TODAY)).is_err());
    }

    #[test]
    fn days_query_defaults_to_last_week() {
        let q = FoodLogDaysQuery { from: None, to: None };
        assert_eq!(q.resolve(date(TODAY)).unwrap(), (date("2026-09-28"), date(TODAY)));
    }

    #[test]
    fn days_query_rejects_inverted_and_oversized_ranges() {
        let q = FoodLogDaysQuery { from: Some(date("2026-10-05")), to: Some(date(TODAY)) };
        assert!(q.resolve(date(TODAY)).is_err());
        let q = FoodLogDaysQuery { from: Some(date("2026-01-01")), to: Some(date(TODAY)) };
        assert!(q.resolve(date(TODAY)).is_err());
        // 剛好 MAX_DAYS_SPAN 天（含頭尾）要收
        let to = date(TODAY);
        let q = FoodLogDaysQuery { from: Some(to - Duration::days(MAX_DAYS_SPAN - 1)), to: Some(to) };
        assert!(q.resolve(to).is_ok());
    }

    #[test]
    fn period_starts() {
        // 2026-10-04 是週日：本週一 = 09-28
        assert_eq!(week_start(date("2026-10-04")), date("2026-09-28"));
        assert_eq!(week_start(date("2026-09-28")), date("2026-09-28"));
        assert_eq!(month_start(date("2026-10-04")), date("2026-10-01"));
    }
}
