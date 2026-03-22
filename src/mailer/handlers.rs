use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::extension::ClientId;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MailError {
    #[error("Failed to build TLS parameters: {0}")]
    Tls(String),
    #[error("Failed to parse address: {0}")]
    ParseAddress(#[from] lettre::address::AddressError),
    #[error("Failed to build email message: {0}")]
    BuildMessage(#[from] lettre::error::Error),
    #[error("SMTP transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[error("Unknown mailer error: {0}")]
    Other(String),
}

pub async fn send_email(
    to_email: String,
    token: String,
    frontend_url: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    smtp_from: String,
) -> Result<(), MailError> {
    tracing::info!(
        "Attempting to send registration email to {} via {}:{}",
        to_email,
        smtp_host,
        smtp_port
    );

    let link = format!("{}/confirm-register?key={}", frontend_url, token);

    let tls_parameters = TlsParameters::builder(smtp_host.clone())
        .build()
        .map_err(|e| MailError::Tls(e.to_string()))?;

    let email = Message::builder()
        .from(smtp_from.parse()?)
        .to(to_email.parse()?)
        .subject("Подтвердите регистрацию")
        .body(format!(
            "Здравствуйте!\n\nПерейдите по ссылке, чтобы завершить регистрацию:\n{}\n\nСсылка действительна 24 часа.",
            link
        ))?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)?
        .port(smtp_port)
        .tls(Tls::Wrapper(tls_parameters))
        .credentials(creds)
        .hello_name(ClientId::Domain("localhost".to_string()))
        .timeout(Some(Duration::from_secs(10)))
        .build();

    match mailer.send(email).await {
        Ok(_) => {
            tracing::info!("Successfully sent registration email to {}", to_email);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to send email to {}: {:?}", to_email, e);
            Err(MailError::Transport(e))
        }
    }
}
