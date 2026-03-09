use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::transport::smtp::extension::ClientId;
use lettre::{Message, SmtpTransport, Transport};

pub fn send_email(
    to_email: String,
    token: String,
    frontend_url: String,
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    smtp_from: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let link = format!("{}/confirm-register?key={}", frontend_url, token);

    let tls_parameters = TlsParameters::builder(smtp_host.clone())
        .build()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let email = Message::builder()
        .from(smtp_from.parse()?)
        .to(to_email.parse()?)
        .subject("Подтвердите регистрацию")
        .body(format!(
            "Здравствуйте!\n\nПерейдите по ссылке, чтобы завершить регистрацию:\n{}\n\nСсылка действительна 24 часа.",
            link
        ))?;

    let creds = Credentials::new(smtp_username, smtp_password);
    let mailer = SmtpTransport::relay(&smtp_host)?
        .port(smtp_port)
        .tls(Tls::Wrapper(tls_parameters))
        .credentials(creds)
        .hello_name(ClientId::Domain("localhost".to_string()))
        .build();

    mailer.send(&email)?;
    Ok(())
}
