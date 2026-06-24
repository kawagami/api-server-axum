use crate::{
    errors::{AppError, RequestError},
    structs::lotto::{Draw, NoteInput, Ticket, TicketListQuery, WinnerRow},
};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

const COLS: &str = "id, member_id, game, draw_date, picks, second, source, \
     checked, prize_tier, notified_at, created_at, updated_at";

/// 批次登錄多注（同一 transaction，全成功或全失敗）
pub async fn create_batch(
    pool: &Pool<Postgres>,
    member_id: i64,
    game: &str,
    draw_date: NaiveDate,
    source: &str,
    notes: &[NoteInput],
) -> Result<Vec<Ticket>, AppError> {
    let mut tx = pool.begin().await?;
    let mut out = Vec::with_capacity(notes.len());
    for n in notes {
        let row: Ticket = sqlx::query_as(&format!(
            "INSERT INTO lotto_tickets (member_id, game, draw_date, picks, second, source)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING {COLS}"
        ))
        .bind(member_id)
        .bind(game)
        .bind(draw_date)
        .bind(&n.picks)
        .bind(n.second)
        .bind(source)
        .fetch_one(&mut *tx)
        .await?;
        out.push(row);
    }
    tx.commit().await?;
    Ok(out)
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &TicketListQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<Ticket>, AppError> {
    let rows = sqlx::query_as(&format!(
        "SELECT {COLS} FROM lotto_tickets
         WHERE member_id = $1
           AND ($2::text IS NULL OR game = $2)
           AND ($3::text IS NULL
                OR ($3 = 'pending' AND checked = false)
                OR ($3 = 'won' AND prize_tier IS NOT NULL)
                OR ($3 = 'lost' AND checked = true AND prize_tier IS NULL))
         ORDER BY created_at DESC
         LIMIT $4 OFFSET $5"
    ))
    .bind(member_id)
    .bind(&query.game)
    .bind(&query.status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_for_member(
    pool: &Pool<Postgres>,
    id: Uuid,
    member_id: i64,
) -> Result<Ticket, AppError> {
    let row: Option<Ticket> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM lotto_tickets WHERE id = $1 AND member_id = $2"
    ))
    .bind(id)
    .bind(member_id)
    .fetch_optional(pool)
    .await?;
    row.ok_or(AppError::RequestError(RequestError::NotFound))
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM lotto_tickets WHERE id = $1 AND member_id = $2")
        .bind(id)
        .bind(member_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::RequestError(RequestError::NotFound));
    }
    Ok(())
}

// ── 通知偏好 ──────────────────────────────────────────────

pub async fn get_member_email(
    pool: &Pool<Postgres>,
    member_id: i64,
) -> Result<Option<String>, AppError> {
    let row: (Option<String>,) = sqlx::query_as("SELECT email FROM members WHERE id = $1")
        .bind(member_id)
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn set_notify_pref(
    pool: &Pool<Postgres>,
    member_id: i64,
    enabled: bool,
) -> Result<(), AppError> {
    sqlx::query("UPDATE members SET lotto_notify_enabled = $1 WHERE id = $2")
        .bind(enabled)
        .bind(member_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── 開獎結果 ──────────────────────────────────────────────

/// upsert 一筆開獎（同 game + draw_date 已存在則略過）
pub async fn upsert_draw(pool: &Pool<Postgres>, draw: &Draw) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO lotto_draws (game, period, draw_date, main_nums, special)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (game, draw_date) DO NOTHING",
    )
    .bind(&draw.game)
    .bind(&draw.period)
    .bind(draw.draw_date)
    .bind(&draw.main_nums)
    .bind(draw.special)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn recent_draws(
    pool: &Pool<Postgres>,
    game: Option<&str>,
    limit: i64,
) -> Result<Vec<Draw>, AppError> {
    let rows = sqlx::query_as(
        "SELECT game, period, draw_date, main_nums, special FROM lotto_draws
         WHERE ($1::text IS NULL OR game = $1)
         ORDER BY draw_date DESC
         LIMIT $2",
    )
    .bind(game)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── 對獎 job 用 ───────────────────────────────────────────

/// 尚未對獎、且已有對應開獎的注：(id, game, picks, second, main_nums, special)
#[allow(clippy::type_complexity)]
pub async fn pending_matches(
    pool: &Pool<Postgres>,
) -> Result<Vec<(Uuid, String, Vec<i16>, Option<i16>, Vec<i16>, i16)>, AppError> {
    let rows = sqlx::query_as(
        "SELECT t.id, t.game, t.picks, t.second, d.main_nums, d.special
         FROM lotto_tickets t
         JOIN lotto_draws d ON d.game = t.game AND d.draw_date = t.draw_date
         WHERE t.checked = false",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_checked(
    pool: &Pool<Postgres>,
    id: Uuid,
    prize_tier: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE lotto_tickets SET checked = true, prize_tier = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(prize_tier)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// 待通知的中獎注（已開啟通知、有 email、尚未寄過）
pub async fn winners_to_notify(pool: &Pool<Postgres>) -> Result<Vec<WinnerRow>, AppError> {
    let rows = sqlx::query_as(
        "SELECT t.id, t.member_id, t.game, t.draw_date, t.prize_tier, m.email
         FROM lotto_tickets t JOIN members m ON m.id = t.member_id
         WHERE t.prize_tier IS NOT NULL
           AND t.notified_at IS NULL
           AND m.lotto_notify_enabled = true
           AND m.email IS NOT NULL
         ORDER BY t.member_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_notified(pool: &Pool<Postgres>, ids: &[Uuid]) -> Result<(), AppError> {
    sqlx::query("UPDATE lotto_tickets SET notified_at = NOW() WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(())
}
