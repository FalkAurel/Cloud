#![allow(unused)]
use std::{env, fmt, time::Duration};

use tracing::info;

use lettre::{
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    address::AddressError,
    error::Error as PrivateEmailError,
    message::{Mailbox, MessageBuilder, MultiPart, SinglePart},
    transport::smtp::{
        Error as PrivateSendError, PoolConfig, authentication::Credentials, response::Response,
    },
};

#[derive(Debug)]
pub enum EmailError {
    EmailError(PrivateEmailError),
    SendError(PrivateSendError),
    AddressError(AddressError),
    InitializationError(&'static str),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::EmailError(e) => write!(f, "Email error: {}", e),
            EmailError::SendError(e) => write!(f, "Send error: {}", e),
            EmailError::AddressError(e) => write!(f, "Address error: {}", e),
            EmailError::InitializationError(e) => write!(f, "Initialization error: {}", e),
        }
    }
}

impl From<PrivateEmailError> for EmailError {
    fn from(value: PrivateEmailError) -> Self {
        Self::EmailError(value)
    }
}

impl From<AddressError> for EmailError {
    fn from(value: AddressError) -> Self {
        Self::AddressError(value)
    }
}

impl From<PrivateSendError> for EmailError {
    fn from(value: PrivateSendError) -> Self {
        Self::SendError(value)
    }
}

const MIN_IDLE_CONNECTION: u8 = 3;
const IDLE_TIMEOUT_MINS: u8 = 40;
const IDLE_TIMEOUT: Duration = Duration::from_mins(IDLE_TIMEOUT_MINS as u64);

pub type EmailSender = AsyncSmtpTransport<Tokio1Executor>;

pub fn init_email_sender() -> Result<EmailSender, EmailError> {
    let host: String = env::var("MAILER_HOST").map_err(|_| {
        EmailError::InitializationError("Missing required environment variable: MAILER_HOST")
    })?;
    let user: String = env::var("MAILER_USER").map_err(|_| {
        EmailError::InitializationError("Missing required environment variable: MAILER_USER")
    })?;
    let password: String = env::var("MAILER_PASSWORD").map_err(|_| {
        EmailError::InitializationError("Missing required environment variable: MAILER_PASSWORD")
    })?;

    info!(
        mailer_host = %host,
        mailer_user = %user,
        min_idle_connections = MIN_IDLE_CONNECTION,
        idle_timeout_mins = IDLE_TIMEOUT_MINS,
        "Establishing SMTP connection pool"
    );

    let email_sender: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
            .unwrap_or_else(|err| panic!("Failed to build SMTP relay for host '{host}': {err}"))
            .credentials(Credentials::new(user, password))
            .pool_config(
                PoolConfig::new()
                    .min_idle(MIN_IDLE_CONNECTION as u32)
                    .idle_timeout(IDLE_TIMEOUT),
            )
            .build();

    info!(mailer_host = %host, "SMTP connection pool ready");

    Ok(email_sender)
}

pub(crate) struct Email<'a> {
    sender: Address,
    receiver: Address,
    subject: Option<&'a str>,
    html_content: Option<&'a str>,
    text_content: Option<&'a str>,
}

impl<'a> Email<'a> {
    pub(crate) fn new(sender: Address, receiver: Address) -> Self {
        Self {
            sender,
            receiver,
            subject: None,
            html_content: None,
            text_content: None,
        }
    }

    pub(crate) fn set_subject(self, subject: &'a str) -> Self {
        Self {
            subject: Some(subject),
            ..self
        }
    }

    pub(crate) fn set_html_content(self, html_content: &'a str) -> Self {
        Self {
            html_content: Some(html_content),
            ..self
        }
    }

    pub(crate) fn set_text_content(self, text_content: &'a str) -> Self {
        Self {
            text_content: Some(text_content),
            ..self
        }
    }

    pub(crate) fn cheap_clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
            subject: self.subject,
            html_content: self.html_content,
            text_content: self.text_content,
        }
    }

    pub(crate) async fn send(self, sender: &EmailSender) -> Result<Response, EmailError> {
        let builder: MessageBuilder = MessageBuilder::new()
            .from(Mailbox::new(None, self.sender))
            .to(Mailbox::new(None, self.receiver))
            .subject(self.subject.unwrap_or(""));

        let multipart: MultiPart = match (self.html_content, self.text_content) {
            (Some(html), Some(text)) => MultiPart::alternative()
                .singlepart(SinglePart::html(html.to_string()))
                .singlepart(SinglePart::plain(text.to_string())),
            (Some(html), None) => {
                MultiPart::alternative().singlepart(SinglePart::html(html.to_owned()))
            }
            (None, Some(text)) => {
                MultiPart::alternative().singlepart(SinglePart::plain(text.to_owned()))
            }
            (None, None) => MultiPart::alternative().singlepart(SinglePart::plain(String::new())),
        };

        let message: Message = builder.multipart(multipart)?;
        Ok(sender.send(message).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDRESS: &str = "falkaurelclouddeployment@gmail.com";
    #[test]
    fn valid_email_accepted() {
        assert!("user@example.com".parse::<Address>().is_ok());
    }

    #[test]
    fn valid_email_with_subdomain_accepted() {
        assert!("user@mail.example.com".parse::<Address>().is_ok());
    }

    #[test]
    fn invalid_email_no_at_rejected() {
        assert!("notanemail".parse::<Address>().is_err());
    }

    #[test]
    fn invalid_email_no_domain_rejected() {
        assert!("user@".parse::<Address>().is_err());
    }

    #[test]
    fn invalid_email_no_local_rejected() {
        assert!("@example.com".parse::<Address>().is_err());
    }

    #[test]
    fn invalid_email_empty_rejected() {
        assert!("".parse::<Address>().is_err());
    }

    #[tokio::test]
    #[ignore = "requires live SMTP relay via MAILER_HOST"]
    async fn send_plain_text_email() {
        let sender: Address = TEST_ADDRESS.parse().unwrap();
        let receiver: Address = TEST_ADDRESS.parse().unwrap();

        let result = Email::new(sender, receiver)
            .set_subject("[Test] Plain text")
            .set_text_content("This is a plain text test email.")
            .send(&init_email_sender().unwrap())
            .await;

        result.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live SMTP relay via MAILER_HOST"]
    async fn send_email_no_subject() {
        let sender: Address = TEST_ADDRESS.parse().unwrap();
        let receiver: Address = TEST_ADDRESS.parse().unwrap();

        Email::new(sender, receiver)
            .set_text_content("Email with no subject set.")
            .send(&init_email_sender().unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live SMTP relay via MAILER_HOST"]
    async fn send_html_email() {
        let sender: Address = TEST_ADDRESS.parse().unwrap();
        let receiver: Address = TEST_ADDRESS.parse().unwrap();

        Email::new(sender, receiver)
            .set_subject("[Test] HTML content")
            .set_html_content(include_str!("../routes/signup_confirmation.html"))
            .send(&init_email_sender().unwrap())
            .await
            .unwrap();
    }
}
