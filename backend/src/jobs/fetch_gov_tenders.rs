use crate::{
    errors::AppError,
    repositories::gov_tenders as repo,
    services::{email, gov_tenders as service},
    state::AppState,
};

/// 每日：依追蹤關鍵字抓政府採購網標案公告 → 寫入新公告 → 寄新標案 email 通知
pub async fn run(state: AppState) {
    super::run_with_retries(
        "fetch_gov_tenders",
        3,
        std::time::Duration::from_secs(1800),
        || fetch_and_notify(&state),
    )
    .await;
}

async fn fetch_and_notify(state: &AppState) -> Result<(), AppError> {
    let pool = state.get_pool();
    let client = state.get_http_client();
    let settings = state.get_settings();

    let keywords: Vec<String> = settings
        .get("gov_tender_keywords")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if keywords.is_empty() {
        return Ok(());
    }

    // 首次執行只建既有公告存量，不把歷史公告當新標案通知
    let seeded = repo::has_any(pool).await?;

    let mut inserted = 0;
    for kw in &keywords {
        for t in service::fetch_by_keyword(client, kw).await? {
            if repo::insert_ignore(pool, &t).await?.is_some() {
                inserted += 1;
            }
        }
    }
    tracing::info!("gov tenders fetched, {} new", inserted);

    let pending = repo::unnotified(pool).await?;
    if pending.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = pending.iter().map(|r| r.id).collect();

    if !seeded {
        return repo::mark_notified(pool, &ids).await;
    }

    let smtp_ready = settings.get("smtp_username").is_some_and(|s| !s.is_empty())
        && settings.get("smtp_password").is_some_and(|s| !s.is_empty());
    if !smtp_ready {
        // 不標 notified，待 SMTP 設定後下輪補寄
        tracing::info!("smtp not configured, skip {} gov tender notifications", pending.len());
        return Ok(());
    }

    let (subject, body) = service::compose_email(&pending);
    email::send_notification(&settings, &subject, body).await;
    repo::mark_notified(pool, &ids).await
}
