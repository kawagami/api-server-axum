use crate::{
    errors::AppError,
    structs::vocab::{BestRun, LeaderboardMe, LeaderboardRow, RunState},
};
use chrono::{DateTime, Utc};
use sqlx::{PgConnection, Pool, Postgres};
use uuid::Uuid;

/// 落地一局結果
/// 落地一局成績。由 caller 持有 transaction —— 必須與發經驗同生同死，否則
/// vocab_runs 有紀錄（會被排行榜聚合）但 member_vocab_exp 沒加，排行榜總和
/// 與玩家等級會長期不一致。
pub async fn insert_run_in_tx(
    conn: &mut PgConnection,
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
    .execute(&mut *conn)
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
/// 發經驗並回新的累計值。由 caller 持有 transaction（理由同 insert_run_in_tx）。
pub async fn upsert_vocab_exp_in_tx(
    conn: &mut PgConnection,
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
    .fetch_one(&mut *conn)
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

/// 週期聚合(exp 總和 + 局數):複習局不計經驗也不計局數
const LEADERBOARD_AGG: &str = "SELECT member_id, SUM(exp_gained)::BIGINT AS exp,
            COUNT(*) AS runs, MAX(ended_at) AS last_at
     FROM vocab_runs
     WHERE language = $1 AND ended_at >= $2 AND mode <> 'review'
     GROUP BY member_id";

/// 週期排行榜 top N:期間內 exp 總和;同分先達成者(最後一局較早)在前
pub async fn leaderboard_top(
    pool: &Pool<Postgres>,
    language: &str,
    from: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<LeaderboardRow>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT RANK() OVER (ORDER BY a.exp DESC) AS rank,
                m.name, m.avatar_url, a.exp, a.runs
         FROM ({LEADERBOARD_AGG}) a
         JOIN members m ON m.id = a.member_id
         ORDER BY a.exp DESC, a.last_at LIMIT $3"
    ))
    .bind(language)
    .bind(from)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// 自己在該週期的名次與 exp;期間內沒打過(計分)局回 None
pub async fn leaderboard_me(
    pool: &Pool<Postgres>,
    language: &str,
    from: DateTime<Utc>,
    member_id: i64,
) -> Result<Option<LeaderboardMe>, AppError> {
    let row = sqlx::query_as(&format!(
        "SELECT rank, exp FROM (
            SELECT member_id, RANK() OVER (ORDER BY exp DESC) AS rank, exp
            FROM ({LEADERBOARD_AGG}) a
         ) r WHERE member_id = $3"
    ))
    .bind(language)
    .bind(from)
    .bind(member_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
