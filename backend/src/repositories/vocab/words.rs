use crate::{errors::AppError, structs::vocab::Word};
use sqlx::{Pool, Postgres};

const WORD_COLS: &str =
    "id, word, part_of_speech, meaning_zh, example_sentence, difficulty, reading, accepted_readings";

/// 依 id 取單字(複習模式指定出題用);已下架回 None
pub async fn word_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<Word>, AppError> {
    let row = sqlx::query_as(&format!(
        "SELECT {WORD_COLS} FROM words WHERE id = $1 AND enabled"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 在難度區間內隨機抽一字,排除本局已出過的;抽不到(題庫被出光)放寬排除重抽
pub async fn random_word(
    pool: &Pool<Postgres>,
    member_id: Option<i64>,
    language: &str,
    min_difficulty: i16,
    max_difficulty: i16,
    exclude_ids: &[i64],
) -> Result<Option<Word>, AppError> {
    // 加權隨機(Efraimidis–Spirakis):key = random()^(1/權重),取最大。
    // 會員「沒測過」(member_word_stats 無 row)權重 4、測過權重 1 → 對應指數 0.25 / 1.0。
    // 訪客 member_id 為 None,LEFT JOIN 全不命中 → 全部權重 4 → 退化成均勻隨機。
    let row: Option<Word> = sqlx::query_as(&format!(
        "SELECT {WORD_COLS} FROM words w
         LEFT JOIN member_word_stats s ON s.word_id = w.id AND s.member_id = $1
         WHERE w.enabled AND w.language = $2
           AND w.difficulty BETWEEN $3 AND $4 AND NOT (w.id = ANY($5))
         ORDER BY power(random(), CASE WHEN s.word_id IS NULL THEN 0.25 ELSE 1.0 END) DESC
         LIMIT 1"
    ))
    .bind(member_id)
    .bind(language)
    .bind(min_difficulty)
    .bind(max_difficulty)
    .bind(exclude_ids)
    .fetch_optional(pool)
    .await?;

    if row.is_some() {
        return Ok(row);
    }

    // 回退:整個難度區間都被本局出光,放寬本局已出過的排除
    let row = sqlx::query_as(&format!(
        "SELECT {WORD_COLS} FROM words
         WHERE enabled AND language = $1 AND difficulty BETWEEN $2 AND $3
         ORDER BY random() LIMIT 1"
    ))
    .bind(language)
    .bind(min_difficulty)
    .bind(max_difficulty)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 該語言題庫的難度上下界(開局查一次,窗口 clamp 用);題庫為空回 None
pub async fn difficulty_bounds(
    pool: &Pool<Postgres>,
    language: &str,
) -> Result<Option<(i16, i16)>, AppError> {
    let (min, max): (Option<i16>, Option<i16>) = sqlx::query_as(
        "SELECT MIN(difficulty), MAX(difficulty) FROM words WHERE language = $1 AND enabled",
    )
    .bind(language)
    .fetch_one(pool)
    .await?;
    Ok(min.zip(max))
}

/// 抽 3 個干擾選項:鄰近難度、排除正解字、排除同釋義
pub async fn distractor_meanings(
    pool: &Pool<Postgres>,
    word_id: i64,
    language: &str,
    difficulty: i16,
    meaning_zh: &str,
) -> Result<Vec<String>, AppError> {
    // language 必濾:跨語言會撞同義釋義(en "eat" 的「吃」對上 ja 食べる)
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT meaning_zh FROM words
         WHERE enabled AND language = $5 AND id <> $1 AND meaning_zh <> $2
           AND difficulty BETWEEN $3 AND $4
         ORDER BY random() LIMIT 3",
    )
    .bind(word_id)
    .bind(meaning_zh)
    .bind((difficulty - 1).max(1))
    .bind((difficulty + 1).min(5))
    .bind(language)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(m,)| m).collect())
}
