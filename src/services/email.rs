use crate::state::AppState;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn send_notification(state: &AppState, subject: &str, body: String) {
    let username = match state.get_setting("smtp_username").filter(|s| !s.is_empty()) {
        Some(v) => v,
        None => return,
    };
    let password = match state.get_setting("smtp_password").filter(|s| !s.is_empty()) {
        Some(v) => v,
        None => return,
    };
    let to = state
        .get_setting("notify_email")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| username.clone());

    match send_email(&username, &password, &to, subject, body).await {
        Ok(_) => tracing::info!("email sent: {}", subject),
        Err(e) => tracing::error!("email fail [{}]: {}", subject, e),
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
