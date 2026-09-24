use crate::{
    errors::AppError,
    structs::vocab::{AdminWord, AdminWordListQuery, UpdateWordRequest},
};
use sqlx::{Pool, Postgres};

// ---------- 後台題庫管理 ----------

const ADMIN_FILTER: &str = "($1::TEXT IS NULL OR w.language = $1)
      AND ($2::SMALLINT IS NULL OR w.difficulty = $2)
      AND ($3::BOOLEAN IS NULL OR w.enabled = $3)
      AND ($4::TEXT IS NULL OR w.word ILIKE '%' || $4 || '%'
           OR w.meaning_zh ILIKE '%' || $4 || '%' OR w.reading ILIKE '%' || $4 || '%')";

/// 後台列表:篩選 + 模糊搜尋 + 全會員答錯統計
pub async fn admin_list(
    pool: &Pool<Postgres>,
    filter: &AdminWordListQuery,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AdminWord>, i64), AppError> {
    let order = match filter.sort.as_deref() {
        Some("wrong") => "wrong_total DESC, w.id",
        _ => "w.id",
    };
    let rows = sqlx::query_as(&format!(
        "SELECT w.id, w.language, w.word, w.reading, w.accepted_readings, w.part_of_speech,
                w.meaning_zh, w.example_sentence, w.difficulty, w.enabled,
                COALESCE(s.wrong_total, 0) AS wrong_total,
                COALESCE(s.correct_total, 0) AS correct_total
         FROM words w
         LEFT JOIN (SELECT word_id, SUM(wrong_count)::BIGINT AS wrong_total,
                           SUM(correct_count)::BIGINT AS correct_total
                    FROM member_word_stats GROUP BY word_id) s ON s.word_id = w.id
         WHERE {ADMIN_FILTER}
         ORDER BY {order} LIMIT $5 OFFSET $6"
    ))
    .bind(&filter.language)
    .bind(filter.difficulty)
    .bind(filter.enabled)
    .bind(&filter.q)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let (total,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM words w WHERE {ADMIN_FILTER}"
    ))
    .bind(&filter.language)
    .bind(filter.difficulty)
    .bind(filter.enabled)
    .bind(&filter.q)
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

/// 取單字語言(更新前驗證用);不存在回 None
pub async fn admin_word_language(
    pool: &Pool<Postgres>,
    id: i64,
) -> Result<Option<String>, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT language FROM words WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(l,)| l))
}

/// 後台更新(表記/語言不可改)
pub async fn admin_update_word(
    pool: &Pool<Postgres>,
    id: i64,
    req: &UpdateWordRequest,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE words SET reading = $2, accepted_readings = $3, part_of_speech = $4,
                meaning_zh = $5, example_sentence = $6, difficulty = $7, enabled = $8,
                updated_at = NOW()
         WHERE id = $1",
    )
    .bind(id)
    .bind(&req.reading)
    .bind(&req.accepted_readings)
    .bind(&req.part_of_speech)
    .bind(&req.meaning_zh)
    .bind(&req.example_sentence)
    .bind(req.difficulty)
    .bind(req.enabled)
    .execute(pool)
    .await?;
    Ok(())
}
