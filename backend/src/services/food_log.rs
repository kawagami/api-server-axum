use std::collections::BTreeMap;

use crate::{
    errors::{AppError, RequestError},
    repositories::food_log as food_log_repo,
    structs::food_log::{
        month_start, week_start, FoodLogDay, FoodLogDayNote, FoodLogDayNoteRequest,
        FoodLogDaysQuery, FoodLogEntry, FoodLogEntryRequest, FoodLogSuggestion, FoodLogSummary,
        MAX_DAY_NOTE_LEN, SUGGESTION_LIMIT, SUGGESTION_LOOKBACK_DAYS,
    },
    utils::date::taipei_today,
};
use chrono::{Duration, NaiveDate};
use sqlx::{Pool, Postgres};

pub async fn list_days(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &FoodLogDaysQuery,
) -> Result<Vec<FoodLogDay>, AppError> {
    let (from, to) = query
        .resolve(taipei_today())
        .map_err(RequestError::UnprocessableContent)?;
    let (entries, notes) = tokio::try_join!(
        food_log_repo::list_entries(pool, member_id, from, to),
        food_log_repo::list_day_notes(pool, member_id, from, to),
    )?;
    Ok(group_days(entries, notes))
}

/// 把（已依日期新到舊、餐別排好的）品項與當天備註併成一天一筆，新到舊。
/// 只有備註、沒有品項的日子也要出現（那天可能只記了「跑步」）。
fn group_days(entries: Vec<FoodLogEntry>, notes: Vec<FoodLogDayNote>) -> Vec<FoodLogDay> {
    let mut days: BTreeMap<NaiveDate, FoodLogDay> = BTreeMap::new();
    let day = |date| FoodLogDay { date, note: None, total: 0, entries: Vec::new() };
    for e in entries {
        let d = days.entry(e.eaten_on).or_insert_with(|| day(e.eaten_on));
        d.total += i64::from(e.amount.unwrap_or(0));
        d.entries.push(e);
    }
    for n in notes {
        days.entry(n.day).or_insert_with(|| day(n.day)).note = Some(n.note);
    }
    days.into_values().rev().collect()
}

fn validate(req: FoodLogEntryRequest) -> Result<crate::structs::food_log::ValidFoodLogEntry, AppError> {
    req.validate(taipei_today())
        .map_err(|e| RequestError::UnprocessableContent(e).into())
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: FoodLogEntryRequest,
) -> Result<FoodLogEntry, AppError> {
    food_log_repo::create(pool, member_id, &validate(req)?).await
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: i64,
    member_id: i64,
    req: FoodLogEntryRequest,
) -> Result<FoodLogEntry, AppError> {
    food_log_repo::update(pool, id, member_id, &validate(req)?).await
}

pub async fn delete(pool: &Pool<Postgres>, id: i64, member_id: i64) -> Result<(), AppError> {
    food_log_repo::delete(pool, id, member_id).await
}

/// 空白備註 = 刪掉那一列（「沒寫」只有一種表示法）
pub async fn set_day_note(
    pool: &Pool<Postgres>,
    member_id: i64,
    day: NaiveDate,
    req: FoodLogDayNoteRequest,
) -> Result<(), AppError> {
    let note = req.note.trim();
    if note.is_empty() {
        return food_log_repo::delete_day_note(pool, member_id, day).await;
    }
    if note.chars().count() > MAX_DAY_NOTE_LEN {
        return Err(RequestError::UnprocessableContent(format!(
            "note 不可超過 {MAX_DAY_NOTE_LEN} 字"
        ))
        .into());
    }
    food_log_repo::upsert_day_note(pool, member_id, day, note).await
}

pub async fn suggestions(pool: &Pool<Postgres>, member_id: i64) -> Result<Vec<FoodLogSuggestion>, AppError> {
    let since = taipei_today() - Duration::days(SUGGESTION_LOOKBACK_DAYS);
    food_log_repo::suggestions(pool, member_id, since, SUGGESTION_LIMIT).await
}

pub async fn summary(pool: &Pool<Postgres>, member_id: i64) -> Result<FoodLogSummary, AppError> {
    let today = taipei_today();
    food_log_repo::summary(pool, member_id, today, week_start(today), month_start(today)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn date(s: &str) -> NaiveDate {
        s.parse().expect("測試日期")
    }

    fn entry(id: i64, eaten_on: &str, amount: Option<i32>) -> FoodLogEntry {
        FoodLogEntry {
            id,
            eaten_on: date(eaten_on),
            meal: "lunch".into(),
            meal_label: None,
            store: None,
            item: format!("item{id}"),
            quantity: 1,
            amount,
            rating: None,
            note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn groups_by_date_newest_first_and_keeps_entry_order() {
        let days = group_days(
            vec![
                entry(3, "2026-10-04", Some(124)),
                entry(1, "2026-10-04", Some(49)),
                entry(2, "2026-10-03", Some(120)),
            ],
            vec![],
        );
        assert_eq!(days.iter().map(|d| d.date).collect::<Vec<_>>(), [date("2026-10-04"), date("2026-10-03")]);
        // 同一天內維持 SQL 給的順序（餐別、id），不在這裡重排
        assert_eq!(days[0].entries.iter().map(|e| e.id).collect::<Vec<_>>(), [3, 1]);
        assert_eq!(days[0].total, 173);
    }

    /// 點數兌換（amount = NULL）不算進合計
    #[test]
    fn total_skips_entries_without_amount() {
        let days = group_days(vec![entry(1, "2026-10-04", Some(45)), entry(2, "2026-10-04", None)], vec![]);
        assert_eq!(days[0].total, 45);
    }

    #[test]
    fn note_only_day_is_included() {
        let days = group_days(
            vec![entry(1, "2026-10-03", Some(120))],
            vec![
                FoodLogDayNote { day: date("2026-10-04"), note: "跑步".into() },
                FoodLogDayNote { day: date("2026-10-03"), note: "換牙刷刷頭".into() },
            ],
        );
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].date, date("2026-10-04"));
        assert!(days[0].entries.is_empty());
        assert_eq!(days[0].note.as_deref(), Some("跑步"));
        assert_eq!(days[1].note.as_deref(), Some("換牙刷刷頭"));
    }
}
