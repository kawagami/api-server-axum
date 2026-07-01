use crate::{
    errors::AppError,
    repositories::lotto as lotto_repo,
    services::lotto::{self, match_draw, PrizeTier, LOTTO649, SUPER638},
    state::AppState,
    structs::lotto::WinnerRow,
};
use chrono::{Datelike, Duration, FixedOffset, Utc};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

/// 每日：抓台彩開獎號碼 → 對未對獎的注比對 → 對已開啟通知的中獎者寄 email
pub async fn run(state: AppState) {
    let pool = state.get_pool().clone();
    let client = state.get_http_client().clone();

    // 1. 抓開獎號碼並寫入（best-effort；抓不到仍用 DB 既有號碼對獎）
    super::run_with_retries(
        "lotto_draws_fetch",
        3,
        std::time::Duration::from_secs(3600),
        || fetch_and_store(&pool, &client),
    )
    .await;

    // 2. 比對 + 通知
    if let Err(e) = check_and_notify(&state, &pool).await {
        tracing::error!("lotto check/notify failed: {}", e);
    }
}

/// 抓當月與上月（跨月邊界保險），upsert 進 lotto_draws
async fn fetch_and_store(pool: &Pool<Postgres>, client: &reqwest::Client) -> Result<(), AppError> {
    for game in [LOTTO649, SUPER638] {
        for month in months_to_fetch() {
            for draw in lotto::fetch_draws(client, game, &month).await? {
                lotto_repo::upsert_draw(pool, &draw).await?;
            }
        }
    }
    Ok(())
}

/// 以台灣時間（UTC+8）算「當月」與「上月」的 YYYY-MM
fn months_to_fetch() -> Vec<String> {
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let now = Utc::now().with_timezone(&tz);
    let cur = format!("{:04}-{:02}", now.year(), now.month());
    let prev_dt = now - Duration::days(now.day() as i64); // 退到上個月
    let prev = format!("{:04}-{:02}", prev_dt.year(), prev_dt.month());
    if prev == cur {
        vec![cur]
    } else {
        vec![cur, prev]
    }
}

async fn check_and_notify(state: &AppState, pool: &Pool<Postgres>) -> Result<(), AppError> {
    for (id, game, picks, second, main, special) in lotto_repo::pending_matches(pool).await? {
        let tier = match_draw(&game, &picks, second, &main, special);
        lotto_repo::mark_checked(pool, id, tier.map(|t| t.as_str())).await?;
    }

    let winners = lotto_repo::winners_to_notify(pool).await?;
    if winners.is_empty() {
        return Ok(());
    }

    let settings = state.get_settings();
    let smtp_ready = settings.get("smtp_username").is_some_and(|s| !s.is_empty())
        && settings.get("smtp_password").is_some_and(|s| !s.is_empty());
    if !smtp_ready {
        // 不標 notified，待 SMTP 設定後下輪補寄
        tracing::info!("smtp not configured, skip {} lotto winner notifications", winners.len());
        return Ok(());
    }

    // 依 member 分組，多注中獎合併一封
    let mut by_member: HashMap<i64, Vec<WinnerRow>> = HashMap::new();
    for w in winners {
        by_member.entry(w.member_id).or_default().push(w);
    }

    let mut notified_ids = Vec::new();
    for (_member_id, rows) in by_member {
        let email = rows[0].email.clone();
        let (subject, body) = compose_email(&rows);
        crate::services::email::send_to(&settings, &email, &subject, body).await;
        notified_ids.extend(rows.iter().map(|r| r.id));
    }

    lotto_repo::mark_notified(pool, &notified_ids).await?;
    Ok(())
}

fn game_label(game: &str) -> &str {
    match game {
        LOTTO649 => "大樂透",
        SUPER638 => "威力彩",
        other => other,
    }
}

fn compose_email(rows: &[WinnerRow]) -> (String, String) {
    let subject = format!("您有 {} 注樂透中獎！", rows.len());

    let mut body = format!("您有 {} 注登錄的樂透中獎：\n\n", rows.len());
    for r in rows {
        let label = PrizeTier::from_db(&r.prize_tier)
            .map(|t| t.label())
            .unwrap_or("中獎");
        body.push_str(&format!("・{} {} — {}\n", game_label(&r.game), r.draw_date, label));
    }
    body.push_str("\n對獎結果僅供參考，實際以台灣彩券公告與兌獎期限為準。");
    (subject, body)
}
