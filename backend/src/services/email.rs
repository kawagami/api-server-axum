use crate::state::Settings;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// 寄信沒送出去的兩種原因。
///
/// **呼叫端只需要知道「這封信到底有沒有送出去」** —— 有 `mark_notified` 這類「標了就
/// 永遠不會再寄」的動作時，一律只在 `Ok` 之後才標。
///
/// 2026-08-05 的實例：`send_to` 當時回傳 `()`，寄失敗只記一行 log 就吞掉，呼叫端照樣
/// `mark_notified` —— 那筆政府標案公告因此**永久遺失**（同一天 SMTP 因容器沒有 IPv6
/// 撞到 `EADDRNOTAVAIL`）。中獎通知走的是一模一樣的路徑。
#[derive(Debug)]
pub enum SendError {
    /// SMTP 未設定。商家 instance 的常態，**刻意不記 log**
    NotConfigured,
    /// 真的寄失敗（已在 `send_to` 內記 ERROR，不必再記一次）
    Failed,
}

/// 寄給後台預設通知信箱（notify_email，未設定則寄件帳號本身）
pub async fn send_notification(
    settings: &Settings,
    subject: &str,
    body: String,
) -> Result<(), SendError> {
    let to = settings
        .get("notify_email")
        .filter(|s| !s.is_empty())
        .or_else(|| settings.get("smtp_username").filter(|s| !s.is_empty()));
    let Some(to) = to else {
        return Err(SendError::NotConfigured);
    };
    send_to(settings, &to, subject, body).await
}

/// 寄給指定收件人；SMTP 未設定回 `NotConfigured`（不記 log，那是設定問題不是故障）
pub async fn send_to(
    settings: &Settings,
    to: &str,
    subject: &str,
    body: String,
) -> Result<(), SendError> {
    let Some(username) = settings.get("smtp_username").filter(|s| !s.is_empty()) else {
        return Err(SendError::NotConfigured);
    };
    let Some(password) = settings.get("smtp_password").filter(|s| !s.is_empty()) else {
        return Err(SendError::NotConfigured);
    };

    match send_email(&username, &password, to, subject, body).await {
        Ok(_) => {
            tracing::info!("email sent: {} -> {}", subject, to);
            Ok(())
        }
        Err(e) => {
            tracing::error!("email fail [{}]: {}", subject, e);
            Err(SendError::Failed)
        }
    }
}

async fn send_email(
    username: &str,
    password: &str,
    to: &str,
    subject: &str,
    body: String,
) -> anyhow::Result<()> {
    let email = Message::builder()
        .from(username.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)?;

    let creds = Credentials::new(username.to_owned(), password.to_owned());

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    Ok(())
}
