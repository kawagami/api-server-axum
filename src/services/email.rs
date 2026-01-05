use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::structs::email::EmailParams;

pub fn send_email_test(params: EmailParams) {
    let username = std::env::var("EMAIL_USERNAME").expect("找不到 EMAIL_USERNAME");
    let password = std::env::var("EMAIL_PASSWORD").expect("找不到 EMAIL_PASSWORD");

    let email = Message::builder()
        .from(Mailbox::new(
            Some("My Rust App".to_owned()),
            username.parse().unwrap(),
        ))
        .to(Mailbox::new(
            Some("我自己".to_owned()),
            username.parse().unwrap(),
        ))
        .subject(params.subject)
        .header(ContentType::TEXT_PLAIN)
        .body(params.body.to_owned())
        .unwrap();

    let creds = Credentials::new(username.to_owned(), password.to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => tracing::info!("✅ Email sent successfully"),
        Err(e) => tracing::error!("❌ Failed to send email : {:?}", e),
    }
}
