use crate::{
    errors::{AppError, RequestError},
    repositories::lotto as lotto_repo,
    services::lotto::{LOTTO649, SUPER638},
    structs::{
        lotto::{
            Draw, DrawListQuery, NoteInput, Ticket, TicketBatchRequest, TicketListQuery, GAMES,
            SOURCES,
        },
        pagination::Paginated,
    },
};
use chrono::{Datelike, NaiveDate, Weekday};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

fn unprocessable(msg: &str) -> AppError {
    RequestError::UnprocessableContent(msg.to_string()).into()
}

/// 各彩種的固定開獎日：大樂透週二/五、威力彩週一/四。
///
/// 這條規則原本只活在前端 `libs/lotto.ts` 的 `DRAW_WEEKDAYS`，後端照單全收。
/// 對獎排程是拿 (game, draw_date) 去 join `lotto_draws`（`pending_matches`），
/// 日期不是開獎日就永遠 join 不到 —— 票會**靜默**停在 pending，
/// 使用者只會覺得「我的票怎麼一直沒對獎」。故在登錄入口就擋掉。
fn draw_weekdays(game: &str) -> &'static [Weekday] {
    match game {
        LOTTO649 => &[Weekday::Tue, Weekday::Fri],
        SUPER638 => &[Weekday::Mon, Weekday::Thu],
        _ => &[],
    }
}

fn game_label(game: &str) -> &'static str {
    match game {
        LOTTO649 => "大樂透",
        SUPER638 => "威力彩",
        _ => "彩券",
    }
}

fn weekday_label(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "一",
        Weekday::Tue => "二",
        Weekday::Wed => "三",
        Weekday::Thu => "四",
        Weekday::Fri => "五",
        Weekday::Sat => "六",
        Weekday::Sun => "日",
    }
}

/// 驗證開獎日。標準週幾直接過；不是標準週幾才查 DB ——
/// 台彩偶爾因國定假日調整開獎日，那些日子 `lotto_draws` 會有實際紀錄，
/// 硬擋週幾會讓真票登錄不進來，所以「DB 裡真的開過」也放行。
async fn validate_draw_date(
    pool: &Pool<Postgres>,
    game: &str,
    draw_date: NaiveDate,
) -> Result<(), AppError> {
    let allowed = draw_weekdays(game);
    if allowed.contains(&draw_date.weekday()) {
        return Ok(());
    }
    if lotto_repo::draw_exists(pool, game, draw_date).await? {
        return Ok(());
    }
    let days: Vec<&str> = allowed.iter().map(|d| weekday_label(*d)).collect();
    Err(unprocessable(&format!(
        "{} 的開獎日為每週{}，{} 不是開獎日",
        game_label(game),
        days.join("、"),
        draw_date
    )))
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
    validate_draw_date(pool, &req.game, req.draw_date).await?;

    lotto_repo::create_batch(pool, member_id, &req.game, req.draw_date, &req.source, &req.notes).await
}

pub async fn list(
    pool: &Pool<Postgres>,
    member_id: i64,
    query: &TicketListQuery,
) -> Result<Paginated<Ticket>, AppError> {
    let page = crate::structs::pagination::PageQuery {
        page: query.page,
        per_page: query.per_page,
    };
    let (limit, offset) = page.to_limit_offset(50);
    // count 與 list 併發跑：序列 await 是白吃一倍延遲（範本同 services/logs.rs）
    let (data, total) = tokio::try_join!(
        lotto_repo::list(pool, member_id, query, limit, offset),
        lotto_repo::count(pool, member_id, query),
    )?;
    Ok(Paginated::new(data, total))
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

#[cfg(test)]
mod tests {
    use super::{draw_weekdays, validate_note, LOTTO649, SUPER638};
    use crate::structs::lotto::NoteInput;
    use chrono::{Datelike, NaiveDate, Weekday};

    fn note(picks: Vec<i16>, second: Option<i16>) -> NoteInput {
        NoteInput { picks, second }
    }

    /// 開獎日對照表是「票會不會被對獎」的唯一依據（排程拿 game+draw_date join lotto_draws），
    /// 寫錯不會有任何錯誤訊息，只會讓票永遠停在 pending，所以釘死在測試裡
    #[test]
    fn draw_weekdays_match_taiwan_lottery_schedule() {
        assert_eq!(draw_weekdays(LOTTO649), &[Weekday::Tue, Weekday::Fri]);
        assert_eq!(draw_weekdays(SUPER638), &[Weekday::Mon, Weekday::Thu]);
        assert!(draw_weekdays("unknown").is_empty());
    }

    /// 2026-08-07 是週五（大樂透開獎日），2026-08-06 是週四（威力彩開獎日）
    #[test]
    fn known_dates_land_on_the_expected_weekday() {
        let fri = NaiveDate::from_ymd_opt(2026, 8, 7).unwrap();
        assert_eq!(fri.weekday(), Weekday::Fri);
        assert!(draw_weekdays(LOTTO649).contains(&fri.weekday()));
        assert!(!draw_weekdays(SUPER638).contains(&fri.weekday()));

        let thu = NaiveDate::from_ymd_opt(2026, 8, 6).unwrap();
        assert_eq!(thu.weekday(), Weekday::Thu);
        assert!(draw_weekdays(SUPER638).contains(&thu.weekday()));
        assert!(!draw_weekdays(LOTTO649).contains(&thu.weekday()));
    }

    #[test]
    fn lotto649_rejects_second_number() {
        assert!(validate_note(LOTTO649, &note(vec![1, 2, 3, 4, 5, 6], None)).is_ok());
        assert!(validate_note(LOTTO649, &note(vec![1, 2, 3, 4, 5, 6], Some(3))).is_err());
        assert!(validate_note(LOTTO649, &note(vec![1, 2, 3, 4, 5, 50], None)).is_err());
        assert!(validate_note(LOTTO649, &note(vec![1, 1, 3, 4, 5, 6], None)).is_err());
    }

    #[test]
    fn super638_requires_second_number_in_range() {
        assert!(validate_note(SUPER638, &note(vec![1, 2, 3, 4, 5, 6], Some(8))).is_ok());
        assert!(validate_note(SUPER638, &note(vec![1, 2, 3, 4, 5, 6], None)).is_err());
        assert!(validate_note(SUPER638, &note(vec![1, 2, 3, 4, 5, 6], Some(9))).is_err());
        assert!(validate_note(SUPER638, &note(vec![1, 2, 3, 4, 5, 39], Some(1))).is_err());
    }
}
