use crate::{
    errors::AppError,
    repositories::invoices as invoices_repo,
    services::invoice_lottery::{self, match_prize, PrizeTier},
    state::AppState,
    structs::invoices::WinnerRow,
};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

/// 每日：抓中獎號碼 → 對該期未對獎發票比對 → 對已開啟通知的中獎者寄 email
pub async fn run(state: AppState) {
    let pool = state.get_pool().clone();
    let client = state.get_http_client().clone();

    // 1. 抓號碼並寫入（best-effort；抓不到仍會用 DB 既有/admin 手動補的號碼對獎）
    super::run_with_retries(
        "invoice_lottery_fetch",
        3,
        std::time::Duration::from_secs(3600),
        || fetch_and_store(&pool, &client),
    )
    .await;

    // 2. 比對 + 通知
    if let Err(e) = check_and_notify(&state, &pool).await {
        tracing::error!("invoice lottery check/notify failed: {}", e);
    }
}

async fn fetch_and_store(pool: &Pool<Postgres>, client: &reqwest::Client) -> Result<(), AppError> {
    for (period, nums) in invoice_lottery::fetch_winning_numbers(client).await? {
        invoices_repo::upsert_period_numbers(pool, &period, &nums).await?;
    }
    Ok(())
}

async fn check_and_notify(state: &AppState, pool: &Pool<Postgres>) -> Result<(), AppError> {
    // 對每個「有號碼且尚有未對獎發票」的期別逐張比對
    for period in invoices_repo::periods_pending_check(pool).await? {
        let nums = invoices_repo::load_period_numbers(pool, &period).await?;
        for (id, number) in invoices_repo::unchecked_by_period(pool, &period).await? {
            let tier = match_prize(&number, &nums);
            invoices_repo::mark_checked(pool, id, tier.map(|t| t.as_str())).await?;
        }
    }

    let winners = invoices_repo::winners_to_notify(pool).await?;
    if winners.is_empty() {
        return Ok(());
    }

    let settings = state.get_settings();
    let smtp_ready = settings.get("smtp_username").is_some_and(|s| !s.is_empty())
        && settings.get("smtp_password").is_some_and(|s| !s.is_empty());
    if !smtp_ready {
        // 不標 notified，待 SMTP 設定後下輪補寄
        tracing::info!("smtp not configured, skip {} lottery winner notifications", winners.len());
        return Ok(());
    }

    // 依 member 分組
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

    invoices_repo::mark_notified(pool, &notified_ids).await?;
    Ok(())
}

fn compose_email(rows: &[WinnerRow]) -> (String, String) {
    let subject = format!("您有 {} 張統一發票中獎！", rows.len());

    let mut body = format!("您有 {} 張登錄的統一發票中獎：\n\n", rows.len());
    for r in rows {
        let (label, amount) = match PrizeTier::from_db(&r.prize_tier) {
            Some(t) => (t.label(), t.amount()),
            None => ("中獎", 0),
        };
        body.push_str(&format!(
            "・期別 {}　發票 {} — {}（NT${}）\n",
            r.period, r.invoice_number, label, amount
        ));
    }
    body.push_str("\n對獎結果僅供參考，實際以財政部公告與兌獎期限為準。");
    (subject, body)
}
