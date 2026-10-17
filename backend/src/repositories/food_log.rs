use crate::{
    errors::{AppError, RequestError},
    structs::food_log::{
        FoodLogDayNote, FoodLogDayTotal, FoodLogEntry, FoodLogItemStat, FoodLogMonthCompare,
        FoodLogMonthlySpend, FoodLogRangeOverview, FoodLogStoreStat, FoodLogSuggestion,
        FoodLogSummary, Meal, ValidFoodLogEntry,
    },
};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres};

const COLS: &str = "id, eaten_on, meal, meal_label, store, item, quantity, amount, rating, note, \
                    created_at, updated_at";

pub async fn list_entries(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<FoodLogEntry>, AppError> {
    // 一天之內依餐別、同餐別依輸入順序（id）
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3
         ORDER BY eaten_on DESC, array_position($4::text[], meal), id"
    ))
    .bind(member_id)
    .bind(from)
    .bind(to)
    .bind(Meal::order())
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_day_notes(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<FoodLogDayNote>, AppError> {
    let rows = sqlx::query_as(
        "SELECT day, note FROM food_log_days
         WHERE member_id = $1 AND day BETWEEN $2 AND $3",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create(
    pool: &Pool<Postgres>,
    member_id: i64,
    e: &ValidFoodLogEntry,
) -> Result<FoodLogEntry, AppError> {
    let row = sqlx::query_as(&format!(
        "INSERT INTO food_log_entries
             (member_id, eaten_on, meal, meal_label, store, item, quantity, amount, rating, note)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING {COLS}"
    ))
    .bind(member_id)
    .bind(e.eaten_on)
    .bind(e.meal.as_str())
    .bind(&e.meal_label)
    .bind(&e.store)
    .bind(&e.item)
    .bind(e.quantity)
    .bind(e.amount)
    .bind(e.rating)
    .bind(&e.note)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update(
    pool: &Pool<Postgres>,
    id: i64,
    member_id: i64,
    e: &ValidFoodLogEntry,
) -> Result<FoodLogEntry, AppError> {
    let row: Option<FoodLogEntry> = sqlx::query_as(&format!(
        "UPDATE food_log_entries
         SET eaten_on = $1, meal = $2, meal_label = $3, store = $4, item = $5,
             quantity = $6, amount = $7, rating = $8, note = $9, updated_at = NOW()
         WHERE id = $10 AND member_id = $11
         RETURNING {COLS}"
    ))
    .bind(e.eaten_on)
    .bind(e.meal.as_str())
    .bind(&e.meal_label)
    .bind(&e.store)
    .bind(&e.item)
    .bind(e.quantity)
    .bind(e.amount)
    .bind(e.rating)
    .bind(&e.note)
    .bind(id)
    .bind(member_id)
    .fetch_optional(pool)
    .await?;

    row.ok_or(AppError::RequestError(RequestError::NotFound))
}

pub async fn delete(pool: &Pool<Postgres>, id: i64, member_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM food_log_entries WHERE id = $1 AND member_id = $2")
        .bind(id)
        .bind(member_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RequestError(RequestError::NotFound));
    }
    Ok(())
}

pub async fn upsert_day_note(
    pool: &Pool<Postgres>,
    member_id: i64,
    day: NaiveDate,
    note: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO food_log_days (member_id, day, note) VALUES ($1, $2, $3)
         ON CONFLICT (member_id, day) DO UPDATE SET note = EXCLUDED.note, updated_at = NOW()",
    )
    .bind(member_id)
    .bind(day)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_day_note(pool: &Pool<Postgres>, member_id: i64, day: NaiveDate) -> Result<(), AppError> {
    sqlx::query("DELETE FROM food_log_days WHERE member_id = $1 AND day = $2")
        .bind(member_id)
        .bind(day)
        .execute(pool)
        .await?;
    Ok(())
}

/// 依 (store, item) 聚合 `since` 之後的紀錄。`store` 為 NULL 的自成一組
/// （GROUP BY / DISTINCT ON 都把 NULL 視為相等，JOIN 那邊要用 `IS NOT DISTINCT FROM` 對上）。
pub async fn suggestions(
    pool: &Pool<Postgres>,
    member_id: i64,
    since: NaiveDate,
    limit: i64,
) -> Result<Vec<FoodLogSuggestion>, AppError> {
    let rows = sqlx::query_as(
        "WITH recent AS (
             SELECT * FROM food_log_entries WHERE member_id = $1 AND eaten_on >= $2
         ), agg AS (
             SELECT store, item,
                    COUNT(*) AS count,
                    MAX(eaten_on) AS last_eaten_on,
                    COUNT(*) FILTER (WHERE meal = 'breakfast')  AS breakfast,
                    COUNT(*) FILTER (WHERE meal = 'lunch')      AS lunch,
                    COUNT(*) FILTER (WHERE meal = 'dinner')     AS dinner,
                    COUNT(*) FILTER (WHERE meal = 'snack')      AS snack,
                    COUNT(*) FILTER (WHERE meal = 'late_night') AS late_night,
                    COUNT(*) FILTER (WHERE meal = 'other')      AS other
             FROM recent GROUP BY store, item
         ), latest AS (
             SELECT DISTINCT ON (store, item) store, item, quantity, amount
             FROM recent ORDER BY store, item, eaten_on DESC, id DESC
         )
         SELECT a.store, a.item, l.quantity AS last_quantity, l.amount AS last_amount,
                a.last_eaten_on, a.count,
                a.breakfast, a.lunch, a.dinner, a.snack, a.late_night, a.other
         FROM agg a
         JOIN latest l ON l.store IS NOT DISTINCT FROM a.store AND l.item = a.item
         ORDER BY a.last_eaten_on DESC, a.count DESC
         LIMIT $3",
    )
    .bind(member_id)
    .bind(since)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 三個期間的花費。`today` 是上界（不算未來，雖然寫入端本來就擋了）。
pub async fn summary(
    pool: &Pool<Postgres>,
    member_id: i64,
    today: NaiveDate,
    week_start: NaiveDate,
    month_start: NaiveDate,
) -> Result<FoodLogSummary, AppError> {
    let row = sqlx::query_as(
        "SELECT COALESCE(SUM(amount) FILTER (WHERE eaten_on = $2), 0)::BIGINT  AS today,
                COALESCE(SUM(amount) FILTER (WHERE eaten_on >= $3), 0)::BIGINT AS week,
                COALESCE(SUM(amount) FILTER (WHERE eaten_on >= $4), 0)::BIGINT AS month
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN LEAST($3, $4) AND $2",
    )
    .bind(member_id)
    .bind(today)
    .bind(week_start)
    .bind(month_start)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// 每天的花費，只有有紀錄的日子（補零在 service 做）
pub async fn daily_totals(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<FoodLogDayTotal>, AppError> {
    let rows = sqlx::query_as(
        "SELECT eaten_on AS date, COALESCE(SUM(amount), 0)::BIGINT AS total
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3
         GROUP BY eaten_on",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 每月的花費與有紀錄的天數，只有有紀錄的月份
pub async fn monthly_totals(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<FoodLogMonthlySpend>, AppError> {
    let rows = sqlx::query_as(
        "SELECT date_trunc('month', eaten_on)::DATE AS month,
                COALESCE(SUM(amount), 0)::BIGINT AS total,
                COUNT(DISTINCT eaten_on) AS recorded_days
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3
         GROUP BY 1",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 本月至今（`this_from`..`today`）與上月同期（`last_from`..`last_to`）的花費
pub async fn month_compare(
    pool: &Pool<Postgres>,
    member_id: i64,
    today: NaiveDate,
    this_from: NaiveDate,
    (last_from, last_to): (NaiveDate, NaiveDate),
) -> Result<FoodLogMonthCompare, AppError> {
    let row = sqlx::query_as(
        "SELECT COALESCE(SUM(amount) FILTER (WHERE eaten_on >= $3), 0)::BIGINT AS this_month,
                COALESCE(SUM(amount) FILTER (WHERE eaten_on BETWEEN $4 AND $5), 0)::BIGINT
                    AS last_month_same_period
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $4 AND $2",
    )
    .bind(member_id)
    .bind(today)
    .bind(this_from)
    .bind(last_from)
    .bind(last_to)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn range_overview(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<FoodLogRangeOverview, AppError> {
    let row = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0)::BIGINT AS total,
                COUNT(DISTINCT eaten_on) AS recorded_days,
                COUNT(*) AS entries,
                COUNT(*) FILTER (WHERE amount IS NULL) AS free_entries
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// 店家排行。一次「去」= 同一天同一餐（同一餐在同一家點好幾樣只算一次）；沒填店家的不列。
pub async fn store_stats(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    limit: i64,
) -> Result<Vec<FoodLogStoreStat>, AppError> {
    let rows = sqlx::query_as(
        "SELECT store,
                COUNT(DISTINCT (eaten_on, meal)) AS visits,
                COALESCE(SUM(amount), 0)::BIGINT AS total,
                MAX(eaten_on) AS last_eaten_on
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3 AND store IS NOT NULL
         GROUP BY store
         ORDER BY visits DESC, total DESC, store
         LIMIT $4",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 品項排行，依 (store, item)；次數是筆數、不乘數量
pub async fn item_stats(
    pool: &Pool<Postgres>,
    member_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    limit: i64,
) -> Result<Vec<FoodLogItemStat>, AppError> {
    let rows = sqlx::query_as(
        "SELECT store, item,
                COUNT(*) AS count,
                COALESCE(SUM(amount), 0)::BIGINT AS total,
                MAX(eaten_on) AS last_eaten_on
         FROM food_log_entries
         WHERE member_id = $1 AND eaten_on BETWEEN $2 AND $3
         GROUP BY store, item
         ORDER BY count DESC, total DESC, item
         LIMIT $4",
    )
    .bind(member_id)
    .bind(from)
    .bind(to)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
