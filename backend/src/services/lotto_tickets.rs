use crate::{
    errors::{AppError, RequestError},
    repositories::lotto as lotto_repo,
    services::lotto::{LOTTO649, SUPER638},
    structs::lotto::{
        Draw, DrawListQuery, NoteInput, Ticket, TicketBatchRequest, TicketListQuery, GAMES, SOURCES,
    },
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

fn unprocessable(msg: &str) -> AppError {
    RequestError::UnprocessableContent(msg.to_string()).into()
}

/// 驗證單注號碼（依彩種）
fn validate_note(game: &str, note: &NoteInput) -> Result<(), AppError> {
    if note.picks.len() != 6 {
        return Err(unprocessable("picks 必須恰好 6 個號碼"));
    }
    let mut sorted = note.picks.clone();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.len() != 6 {
        return Err(unprocessable("picks 不可有重複號碼"));
    }

    match game {
        LOTTO649 => {
            if note.picks.iter().any(|&n| !(1..=49).contains(&n)) {
                return Err(unprocessable("大樂透號碼須介於 1~49"));
            }
            if note.second.is_some() {
                return Err(unprocessable("大樂透不可帶 second"));
            }
        }
        SUPER638 => {
            if note.picks.iter().any(|&n| !(1..=38).contains(&n)) {
                return Err(unprocessable("威力彩第一區號碼須介於 1~38"));
            }
            match note.second {
                Some(s) if (1..=8).contains(&s) => {}
                Some(_) => return Err(unprocessable("威力彩第二區號碼須介於 1~8")),
                None => return Err(unprocessable("威力彩必須帶 second（第二區號碼）")),
            }
        }
        _ => unreachable!("game 已於 register 驗證"),
    }
    Ok(())
}

/// 批次登錄；全批驗證後全批寫入（任一注不合法整批退回）
pub async fn register(
    pool: &Pool<Postgres>,
    member_id: i64,
    req: &TicketBatchRequest,
) -> Result<Vec<Ticket>, AppError> {
    if !GAMES.contains(&req.game.as_str()) {
        return Err(unprocessable("game 必須為 lotto649 或 super_lotto638"));
    }
    if !SOURCES.contains(&req.source.as_str()) {
        return Err(unprocessable("source 必須為 qr 或 manual"));
    }
    if req.notes.is_empty() {
        return Err(unprocessable("notes 不可為空"));
    }
    for note in &req.notes {
        validate_note(&req.game, note)?;
    }

    lotto_repo::create_batch(pool, member_id, &req.game, req.draw_date, &req.source, &req.notes).await
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &TicketListQuery,
) -> Result<Vec<Ticket>, AppError> {
    let page = crate::structs::pagination::PageQuery {
        page: query.page,
        per_page: query.per_page,
    };
    let (limit, offset) = page.to_limit_offset(50);
    lotto_repo::list(pool, member_id, query, limit, offset).await
}

pub async fn get(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<Ticket, AppError> {
    lotto_repo::get_for_member(pool, id, member_id).await
}

pub async fn delete(pool: &Pool<Postgres>, id: Uuid, member_id: i64) -> Result<(), AppError> {
    lotto_repo::delete(pool, id, member_id).await
}

pub async fn draws(pool: &Pool<Postgres>, query: &DrawListQuery) -> Result<Vec<Draw>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    lotto_repo::recent_draws(pool, query.game.as_deref(), limit).await
}

/// 開關中獎 email 通知；開啟須有 email
pub async fn set_notify(
    pool: &Pool<Postgres>,
    member_id: i64,
    enabled: bool,
) -> Result<bool, AppError> {
    if enabled {
        let email = lotto_repo::get_member_email(pool, member_id).await?;
        if email.filter(|e| !e.is_empty()).is_none() {
            return Err(unprocessable("此帳號未綁定 email，無法開啟中獎通知"));
        }
    }
    lotto_repo::set_notify_pref(pool, member_id, enabled).await?;
    Ok(enabled)
}
