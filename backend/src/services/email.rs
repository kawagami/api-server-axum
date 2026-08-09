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

    // 總嘗試次數（含第一次）。理由同 `utils::reqwest::send_retrying`：這台機器對外連線
    // 偶爾會在 connect 階段抖一下，而通知信失敗的代價是「使用者不知道自己中獎」。
    const ATTEMPTS: u32 = 3;
    const BACKOFF_MS: u64 = 300;

    let mut attempt = 1;
    loop {
        match send_email(&username, &password, to, subject, body.clone()).await {
            Ok(_) => {
                tracing::info!("email sent: {} -> {}", subject, to);
                return Ok(());
            }
            Err(e) => {
                if attempt >= ATTEMPTS || !is_connection_failure(&e) {
                    tracing::error!("email fail [{}]: {}", subject, e);
                    return Err(SendError::Failed);
                }
                tracing::warn!(
                    "寄信連線失敗（第 {}/{} 次），{}ms 後重試 [{}]: {}",
                    attempt,
                    ATTEMPTS,
                    BACKOFF_MS * attempt as u64,
                    subject,
                    e
                );
                tokio::time::sleep(std::time::Duration::from_millis(
                    BACKOFF_MS * attempt as u64,
                ))
                .await;
                attempt += 1;
            }
        }
    }
}

/// 只有「**連線階段**」的失敗才重試。
///
/// 訊息一旦進了 SMTP 對話（對方回了狀態碼、或連線在 DATA 之後斷掉），重試就可能讓
/// 同一封信寄兩次 —— 對中獎通知來說那比慢一輪更糟。判斷方式是往 source chain 找
/// `io::Error`，只認那幾種「還沒接上」的 kind（`is_timeout` 也是這樣實作的）。
fn is_connection_failure(err: &anyhow::Error) -> bool {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(err.as_ref());
    while let Some(e) = source {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            return matches!(
                io.kind(),
                std::io::ErrorKind::AddrNotAvailable      // 這次撞的就是這個（os error 99）
                    | std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::NetworkUnreachable
                    | std::io::ErrorKind::HostUnreachable
                    | std::io::ErrorKind::TimedOut
            );
        }
        source = e.source();
    }
    false
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
