use crate::{
    errors::AppError,
    structs::vocab::{BestRun, MistakeEntry, RunState, Word},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

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

/// 錯題本:答錯過(wrong_count > 0)的字,未掌握(答錯 > 答對)的排前面
pub async fn mistakes(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
) -> Result<Vec<MistakeEntry>, AppError> {
    let rows = sqlx::query_as(
        "SELECT w.word, w.part_of_speech, w.meaning_zh, w.reading, w.difficulty,
                s.wrong_count, s.correct_count, s.last_seen_at
         FROM member_word_stats s JOIN words w ON w.id = s.word_id
         WHERE s.member_id = $1 AND w.language = $2 AND s.wrong_count > 0
         ORDER BY (s.correct_count >= s.wrong_count), s.wrong_count DESC, s.last_seen_at DESC",
    )
    .bind(member_id)
    .bind(language)
    .fetch_all(pool)
    .await?;
    Ok(rows)
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

/// 落地一局結果
pub async fn insert_run(
    pool: &Pool<Postgres>,
    run_id: Uuid,
    member_id: i64,
    state: &RunState,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO vocab_runs
            (id, member_id, answered_count, correct_count, max_combo, exp_gained, started_at, mode, language)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(run_id)
    .bind(member_id)
    .bind(state.answered)
    .bind(state.correct)
    .bind(state.max_combo)
    .bind(state.exp)
    .bind(state.started_at)
    .bind(state.mode.as_str())
    .bind(state.language.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

/// 指定模式的最佳紀錄(新紀錄判定用)
pub async fn best_run(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
    mode: &str,
) -> Result<Option<BestRun>, AppError> {
    let row = sqlx::query_as(
        "SELECT mode, correct_count, max_combo, exp_gained FROM vocab_runs
         WHERE member_id = $1 AND language = $2 AND mode = $3
         ORDER BY correct_count DESC, max_combo DESC LIMIT 1",
    )
    .bind(member_id)
    .bind(language)
    .bind(mode)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// 各模式的最佳紀錄(每模式一筆)
pub async fn bests(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
) -> Result<Vec<BestRun>, AppError> {
    let rows = sqlx::query_as(
        "SELECT DISTINCT ON (mode) mode, correct_count, max_combo, exp_gained
         FROM vocab_runs WHERE member_id = $1 AND language = $2
         ORDER BY mode, correct_count DESC, max_combo DESC",
    )
    .bind(member_id)
    .bind(language)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 加分語言經驗值,回傳加完後該語言總 exp(members.exp 已凍結,不再讀寫)
pub async fn upsert_vocab_exp(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
    delta: i64,
) -> Result<i64, AppError> {
    let (exp,): (i64,) = sqlx::query_as(
        "INSERT INTO member_vocab_exp (member_id, language, exp) VALUES ($1, $2, $3)
         ON CONFLICT (member_id, language) DO UPDATE
            SET exp = member_vocab_exp.exp + EXCLUDED.exp
         RETURNING exp",
    )
    .bind(member_id)
    .bind(language)
    .bind(delta)
    .fetch_one(pool)
    .await?;
    Ok(exp)
}

/// 該語言的總經驗;沒玩過(無列)為 0
pub async fn vocab_exp(
    pool: &Pool<Postgres>,
    member_id: i64,
    language: &str,
) -> Result<i64, AppError> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT exp FROM member_vocab_exp WHERE member_id = $1 AND language = $2")
            .bind(member_id)
            .bind(language)
            .fetch_optional(pool)
            .await?;
    Ok(row.map_or(0, |(exp,)| exp))
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
