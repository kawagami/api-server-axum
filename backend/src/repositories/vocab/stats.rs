use crate::{
    errors::AppError,
    structs::vocab::{MistakeEntry, MistakeListQuery},
};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres};

/// 錯題本的共用篩選:$1 member、$2 language、$3 只看未掌握、$4 模糊搜尋
const MISTAKE_FILTER: &str = "s.member_id = $1 AND w.language = $2 AND s.wrong_count > 0
      AND ($3::BOOLEAN IS NOT TRUE OR s.wrong_count > s.correct_count)
      AND ($4::TEXT IS NULL OR w.word ILIKE '%' || $4 || '%'
           OR w.meaning_zh ILIKE '%' || $4 || '%' OR w.reading ILIKE '%' || $4 || '%')";

/// 錯題本一頁:答錯過(wrong_count > 0)的字,排序由 query 指定
///
/// 一定要分頁 —— 錯題本隨學習無界成長,全撈會讓 SSR payload 跟著使用者的學習量膨脹。
pub async fn mistakes(
    pool: &Pool<Postgres>,
    member_id: i64,
    q: &MistakeListQuery,
) -> Result<Vec<MistakeEntry>, AppError> {
    let order_by = q.sort.order_by();
    let rows = sqlx::query_as(&format!(
        "SELECT w.word, w.part_of_speech, w.meaning_zh, w.reading, w.difficulty,
                s.wrong_count, s.correct_count, s.last_seen_at
         FROM member_word_stats s JOIN words w ON w.id = s.word_id
         WHERE {MISTAKE_FILTER}
         ORDER BY {order_by} LIMIT $5 OFFSET $6"
    ))
    .bind(member_id)
    .bind(q.language.as_str())
    .bind(q.unmastered)
    .bind(q.search())
    .bind(q.limit())
    .bind(q.offset())
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// (符合篩選的錯字數, 全部未掌握字數)
///
/// 兩個數字一起算:`total` 給錯題本分頁,`reviewable` 給複習按鈕
/// —— 後者不能被錯題本的搜尋條件影響,所以走各自的 FILTER。
pub async fn mistake_counts(
    pool: &Pool<Postgres>,
    member_id: i64,
    q: &MistakeListQuery,
) -> Result<(i64, i64), AppError> {
    let (total, reviewable): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE
                    ($3::BOOLEAN IS NOT TRUE OR s.wrong_count > s.correct_count)
                    AND ($4::TEXT IS NULL OR w.word ILIKE '%' || $4 || '%'
                         OR w.meaning_zh ILIKE '%' || $4 || '%'
                         OR w.reading ILIKE '%' || $4 || '%'))::BIGINT,
                COUNT(*) FILTER (WHERE s.wrong_count > s.correct_count)::BIGINT
         FROM member_word_stats s JOIN words w ON w.id = s.word_id
         WHERE s.member_id = $1 AND w.language = $2 AND s.wrong_count > 0",
    )
    .bind(member_id)
    .bind(q.language.as_str())
    .bind(q.unmastered)
    .bind(q.search())
    .fetch_one(pool)
    .await?;
    Ok((total, reviewable))
}

/// 近期有玩過的日子(台北日,新到舊,去重);連續天數計算用
///
/// 用固定 +8 小時偏移而非 `AT TIME ZONE`,與 `utils::date::taipei_offset()` 同語意
/// (台灣 1980 起無日光節約,固定偏移即正確)。
pub async fn played_days(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
    within_days: i32,
) -> Result<Vec<NaiveDate>, AppError> {
    let rows: Vec<(NaiveDate,)> = sqlx::query_as(
        "SELECT DISTINCT ((ended_at + INTERVAL '8 hours') AT TIME ZONE 'UTC')::date AS day
         FROM vocab_runs
         WHERE member_id = $1 AND language = $2
           AND ended_at >= NOW() - make_interval(days => $3)
         ORDER BY day DESC",
    )
    .bind(member_id)
    .bind(language)
    .bind(within_days)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(d,)| d).collect())
}

/// 複習出題池:尚未掌握的錯字(答錯次數 > 答對次數),錯最多的優先
pub async fn review_word_ids(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
    limit: i64,
) -> Result<Vec<i64>, AppError> {
    let rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT s.word_id
         FROM member_word_stats s JOIN words w ON w.id = s.word_id
         WHERE s.member_id = $1 AND w.language = $2
           AND s.wrong_count > s.correct_count AND w.enabled
         ORDER BY s.wrong_count DESC, s.last_seen_at DESC
         LIMIT $3",
    )
    .bind(member_id)
    .bind(language)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// 在指定字集中,現已掌握(答對次數 >= 答錯次數)的數量
pub async fn count_mastered_among(
    pool: &Pool<Postgres>,
    member_id: i64,
    word_ids: &[i64],
) -> Result<i64, AppError> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM member_word_stats
         WHERE member_id = $1 AND word_id = ANY($2)
           AND correct_count >= wrong_count AND wrong_count > 0",
    )
    .bind(member_id)
    .bind(word_ids)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// 逐題更新學習進度
pub async fn upsert_word_stat(
    pool: &Pool<Postgres>,
    member_id: i64,
    word_id: i64,
    correct: bool,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO member_word_stats (member_id, word_id, correct_count, wrong_count)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (member_id, word_id) DO UPDATE SET
            correct_count = member_word_stats.correct_count + EXCLUDED.correct_count,
            wrong_count = member_word_stats.wrong_count + EXCLUDED.wrong_count,
            last_seen_at = NOW()",
    )
    .bind(member_id)
    .bind(word_id)
    .bind(if correct { 1 } else { 0 })
    .bind(if correct { 0 } else { 1 })
    .execute(pool)
    .await?;
    Ok(())
}

/// (總局數, 答對過的單字數)
pub async fn member_stats(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
) -> Result<(i64, i64), AppError> {
    let (total_runs,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM vocab_runs WHERE member_id = $1 AND language = $2")
            .bind(member_id)
            .bind(language)
            .fetch_one(pool)
            .await?;
    let (words_learned,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM member_word_stats s JOIN words w ON w.id = s.word_id
         WHERE s.member_id = $1 AND w.language = $2 AND s.correct_count > 0",
    )
    .bind(member_id)
    .bind(language)
    .fetch_one(pool)
    .await?;
    Ok((total_runs, words_learned))
}
