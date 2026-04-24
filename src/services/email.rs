use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::{
    errors::{AppError, SystemError},
    structs::email::EmailParams,
};

pub fn send_email_test(params: EmailParams) -> Result<(), AppError> {
    let username = std::env::var("EMAIL_USERNAME").expect("找不到 EMAIL_USERNAME");
    let password = std::env::var("EMAIL_PASSWORD").expect("找不到 EMAIL_PASSWORD");

    let addr = username
        .parse()
        .map_err(|e: lettre::address::AddressError| {
            AppError::SystemError(SystemError::Internal(format!("EMAIL_USERNAME 格式無效: {}", e)))
        })?;

    let email = Message::builder()
        .from(Mailbox::new(Some("My Rust App".to_owned()), addr))
        .to(Mailbox::new(
            Some("我自己".to_owned()),
            username
                .parse()
                .map_err(|e: lettre::address::AddressError| {
                    AppError::SystemError(SystemError::Internal(format!(
                        "EMAIL_USERNAME 格式無效: {}",
                        e
                    )))
                })?,
        ))
        .subject(params.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(params.body.to_owned())
        .map_err(|e| AppError::SystemError(SystemError::Internal(format!("email 建構失敗: {}", e))))?;

    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .map_err(|e| AppError::SystemError(SystemError::Internal(format!("SMTP relay 建立失敗: {}", e))))?
        .credentials(Credentials::new(username, password))
        .build();

    mailer
        .send(&email)
        .map_err(|e| AppError::SystemError(SystemError::Internal(format!("email 發送失敗: {}", e))))?;

    tracing::info!("Email sent successfully");
    Ok(())
}
