use std::{env, str::Split, sync::LazyLock, time::Duration};

use tracing::info;

use email_syntax_verify_opt::validate_email;
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
pub(crate) enum EmailError {
    EmailError(PrivateEmailError),
    SendError(PrivateSendError),
    AddressError(AddressError),
    InitializationError(&'static str),
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

pub(crate) struct ValidatedEmail<'a>(&'a str);

impl<'a> ValidatedEmail<'a> {
    pub(crate) fn new(email: &'a str) -> Option<Self> {
        if validate_email(email) {
            Some(Self(email))
        } else {
            None
        }
    }
}

pub(crate) struct Email<'a> {
    sender: ValidatedEmail<'a>,
    receiver: ValidatedEmail<'a>,
    subject: Option<&'a str>,
    html_content: Option<&'a str>,
    text_content: Option<&'a str>,
}

impl<'a> Email<'a> {
    pub(crate) fn new(sender: ValidatedEmail<'a>, receiver: ValidatedEmail<'a>) -> Self {
        Self {
            sender,
            receiver,
            subject: None,
            html_content: None,
            text_content: None,
        }
    }

    fn get_address(validated_email: ValidatedEmail) -> Address {
        let mut split: Split<'_, &str> = validated_email.0.split("@");

        let user: &str = split.next().unwrap();
        let domain: &str = split.next().unwrap();

        Address::new_dangerous(user, domain)
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

    pub(crate) async fn send(self, sender: &EmailSender) -> Result<Response, EmailError> {
        let builder: MessageBuilder = MessageBuilder::new()
            .from(Mailbox::new(None, Self::get_address(self.sender)))
            .to(Mailbox::new(None, Self::get_address(self.receiver)))
            .subject(self.subject.unwrap_or(""));

        let multipart: MultiPart = match (self.html_content, self.text_content) {
            (Some(html), Some(text)) => MultiPart::related()
                .singlepart(SinglePart::html(html.to_string()))
                .singlepart(SinglePart::plain(text.to_string())),
            (Some(html), None) => {
                MultiPart::related().singlepart(SinglePart::html(html.to_string()))
            }
            (None, Some(text)) => {
                MultiPart::related().singlepart(SinglePart::plain(text.to_string()))
            }
            (None, None) => MultiPart::related().singlepart(SinglePart::plain(String::new())),
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
        assert!(ValidatedEmail::new("user@example.com").is_some());
    }

    #[test]
    fn valid_email_with_subdomain_accepted() {
        assert!(ValidatedEmail::new("user@mail.example.com").is_some());
    }

    #[test]
    fn invalid_email_no_at_rejected() {
        assert!(ValidatedEmail::new("notanemail").is_none());
    }

    #[test]
    fn invalid_email_no_domain_rejected() {
        assert!(ValidatedEmail::new("user@").is_none());
    }

    #[test]
    fn invalid_email_no_local_rejected() {
        assert!(ValidatedEmail::new("@example.com").is_none());
    }

    #[test]
    fn invalid_email_empty_rejected() {
        assert!(ValidatedEmail::new("").is_none());
    }

    #[tokio::test]
    #[ignore = "requires live SMTP relay via MAILER_HOST"]
    async fn send_plain_text_email() {
        let sender: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();
        let receiver: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();

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
        let sender: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();
        let receiver: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();

        Email::new(sender, receiver)
            .set_text_content("Email with no subject set.")
            .send(&init_email_sender().unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires live SMTP relay via MAILER_HOST"]
    async fn send_html_email() {
        let sender: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();
        let receiver: ValidatedEmail = ValidatedEmail::new(TEST_ADDRESS).unwrap();

        Email::new(sender, receiver)
            .set_subject("[Test] HTML content")
            .set_html_content(include_str!("../routes/signup_confirmation.html"))
            .send(&init_email_sender().unwrap())
            .await
            .unwrap();
    }
}
