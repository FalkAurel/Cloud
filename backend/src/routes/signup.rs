use crate::ARGON_2;
use crate::data_definitions::{FixedSizedStr, UserCreationView};
use crate::data_definitions::{MAX_UTF8_BYTES, UserSignupRequest};
use crate::database::user_repository::UserRepository;
use crate::database::{ReadOnly, Transactional};
use argon2::PasswordHasher;
use argon2::password_hash::{SaltString, rand_core::OsRng};
use rocket::http::Status;
use rocket::tokio::task::JoinHandle;
use rocket::serde::json::Json;
use rocket::{State, post, tokio};
use sqlx::Transaction;
use sqlx::{MySql, Pool};
use tracing::{error, warn};

const MIN_PASSWORD_LENGTH: u8 = 8;
const MAX_PASSWORD_LENGTH: u8 = MAX_UTF8_BYTES as u8;

#[cfg(feature = "email")]
use email_const::*;

#[cfg(feature = "email")]
mod email_const {
    use super::{MySql, Pool, error, warn};
    pub(super) use crate::data_definitions::{Email, EmailError, EmailSender};
    pub(super) use lettre::Address;
    pub(super) use lettre::transport::smtp::response::{Response, Severity};
    use rocket::http::Status;
    pub(super) use rocket::tokio::time::sleep;
    use sqlx::Error as SQLError;
    pub(super) use std::num::NonZero;
    use std::time::Duration;
    use tracing::info;

    pub(super) const RETRY_WAIT_TIME: Duration = Duration::from_secs(30);
    pub(super) const RETRIES: Option<NonZero<u8>> = NonZero::new(3);
    pub(super) const SIGN_UP_SUBJECT: &str = "Welcome to your own Cloud – You're all set!";
    pub(super) const SIGN_UP_HTML: &str = include_str!("signup_confirmation.html");

    pub(super) const DELETE_USER_QUERY_STR: &str = r#"
    DELETE LOW_PRIORITY FROM users WHERE email = ?;
    "#;

    pub(super) async fn handle_signup_email(
        email_sender: EmailSender,
        sender_address: Address,
        email_address: Address,
        raw_email: String,
        db: Pool<MySql>,
    ) -> Result<(), (Status, &'static str)> {
        let email: Email = Email::new(sender_address, email_address)
            .set_subject(SIGN_UP_SUBJECT)
            .set_html_content(SIGN_UP_HTML);

        match send_email(&email_sender, email, RETRIES).await {
            Ok(Ok(_)) => {
                info!("User signed up successfully");
                Ok(())
            }
            Ok(Err(response)) => {
                warn!(code = ?response.code(), "Email failed to send, reverting signup");
                if let Err(err) = revert_signup(&raw_email, &db).await {
                    error!(error=%err, "Failed to revert signup after email failure");
                }
                Err((Status::InternalServerError, "Failed to send confirmation email"))
            }
            Err(err) => {
                error!(error = %err, "Failed to send email, reverting signup");
                if let Err(err) = revert_signup(&raw_email, &db).await {
                    error!(error=%err, "Failed to revert signup after email failure");
                }
                Err((Status::InternalServerError, "Failed to send confirmation email"))
            }
        }
    }

    pub(super) async fn revert_signup(email: &str, db: &Pool<MySql>) -> Result<(), SQLError> {
        sqlx::query(DELETE_USER_QUERY_STR)
            .bind(email)
            .execute(db)
            .await?;

        Ok(())
    }

    pub(super) async fn send_email(
        email_sender: &EmailSender,
        email: Email<'_>,
        retries: Option<NonZero<u8>>,
    ) -> Result<Result<Response, Response>, EmailError> {
        let attempts: u8 = retries.map_or(1, |r| r.get() + 1);

        for attempt in 0..attempts {
            info!("Sending email");
            match email.clone().send(email_sender).await {
                Ok(response) => match response.code().severity {
                    Severity::TransientNegativeCompletion => {
                        let error: &str = response.message().next().unwrap_or("Unknown Error");
                        if attempt < attempts - 1 {
                            warn!(
                                remaining = attempts - attempt - 1,
                                error, "Transient failure, retrying..."
                            );
                            sleep(RETRY_WAIT_TIME).await;
                            continue;
                        }
                        warn!(code = ?response.code(), error, "Transient failure, retries exhausted");
                        return Ok(Err(response));
                    }
                    Severity::PermanentNegativeCompletion => {
                        warn!(code = ?response.code(), "Permanent email failure");
                        return Ok(Err(response));
                    }
                    Severity::PositiveCompletion | Severity::PositiveIntermediate => {
                        info!(code = ?response.code(), "Email sent successfully");
                        return Ok(Ok(response));
                    }
                },
                Err(err) => {
                    error!(error = %err, "Failed to send email");
                    return Err(err);
                }
            }
        }

        unreachable!()
    }
}

/// Hashes `password` and writes the new user row inside a committed transaction.
/// Input lengths must be validated before calling this function.
async fn persist_user(
    name: &str,
    email: &str,
    password: &str,
    db: &Pool<MySql>,
) -> Result<(), (Status, &'static str)> {
    let name_fixed: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str(name).map_err(|err| {
        error!(error = %err, "Signup: name exceeds max length after validation");
        (Status::InternalServerError, "Internal server error")
    })?;

    let email_fixed: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str(email).map_err(|err| {
        error!(error = %err, "Signup: email exceeds max length after validation");
        (Status::InternalServerError, "Internal server error")
    })?;

    let salt_string: SaltString = SaltString::generate(&mut OsRng);
    let password: String = password.to_owned();
    let handle: JoinHandle<Result<String, (Status, &'static str)>> =
        tokio::task::spawn_blocking(move || {
            match ARGON_2.hash_password(password.as_bytes(), &salt_string) {
                Ok(hash) => Ok(hash.to_string()),
                Err(err) => {
                    error!(error = %err, "Failed to hash password");
                    Err((Status::InternalServerError, "Failed to hash password"))
                }
            }
        });

    let hashed_password: String = handle.await.or_else(|err| {
        warn!(error = %err, "Hashing task failed");
        Err((Status::InternalServerError, "Internal server error"))
    })??;

    let hashed_password = FixedSizedStr::new_from_str(&hashed_password).map_err(|err| {
        error!(error = %err, "Signup: hashed password exceeds max length");
        (Status::InternalServerError, "Internal server error")
    })?;

    let mut transaction: Transaction<MySql> = db.begin().await.map_err(|err| {
        error!(error = %err, "Signup: could not begin transaction");
        (Status::InternalServerError, "Internal server error")
    })?;

    let user_creation_view = UserCreationView::new(&name_fixed, &email_fixed);
    let create_user = UserRepository::create(&user_creation_view, &hashed_password);

    if let Err(err) = create_user.execute(&mut transaction).await {
        error!(error = %err, "Signup: database error");
        return Err((Status::InternalServerError, "Internal server error"));
    }

    transaction.commit().await.map_err(|err| {
        error!(error = %err, "Signup: could not commit transaction");
        (Status::InternalServerError, "Internal server error")
    })?;

    Ok(())
}

#[cfg(feature = "email")]
#[post("/signup", format = "json", data = "<signup_request>")]
pub async fn signup(
    signup_request: Json<UserSignupRequest<'_>>,
    db: &State<Pool<MySql>>,
    email_sender: &State<EmailSender>,
    sender_address: &State<Address>,
) -> Result<Status, (Status, &'static str)> {
    if !verify_password_length(signup_request.password) {
        return Err((Status::BadRequest, "Password length is invalid"));
    }
    if !verify_username_length(signup_request.name) {
        return Err((Status::BadRequest, "Username length is invalid"));
    }
    if !verify_username_content(signup_request.name) {
        return Err((Status::BadRequest, "Username contains invalid characters"));
    }
    if !verify_email_length(signup_request.email) {
        return Err((Status::BadRequest, "Email length is invalid"));
    }

    let validated_email: Address = match signup_request.email.parse() {
        Ok(email) => email,
        Err(err) => {
            warn!(error = %err, "Signup failed: invalid email address");
            return Err((Status::BadRequest, "Invalid email address"));
        }
    };

    match UserRepository::email_exists(validated_email.as_ref())
        .read(db)
        .await
    {
        Ok(true) => {
            warn!("Signup failed: email already in use");
            return Err((Status::Conflict, "Email already in use"));
        }
        Err(err) => {
            error!(error = %err, "Signup failed: could not check email uniqueness");
            return Err((Status::InternalServerError, "Internal server error"));
        }
        _ => (),
    }

    persist_user(
        signup_request.name,
        signup_request.email,
        signup_request.password,
        db,
    )
    .await?;

    let sender = email_sender.inner().clone();
    let from_address = sender_address.inner().clone();
    let pool = db.inner().clone();
    let raw_email = signup_request.email.to_owned();
    handle_signup_email(sender, from_address, validated_email, raw_email, pool).await?;

    Ok(Status::Created)
}

#[cfg(not(feature = "email"))]
#[post("/signup", format = "json", data = "<signup_request>")]
pub async fn signup(
    signup_request: Json<UserSignupRequest<'_>>,
    db: &State<Pool<MySql>>,
) -> Result<Status, (Status, &'static str)> {
    if !verify_password_length(signup_request.password) {
        return Err((Status::BadRequest, "Password length is invalid"));
    }
    if !verify_username_length(signup_request.name) {
        return Err((Status::BadRequest, "Username length is invalid"));
    }
    if !verify_username_content(signup_request.name) {
        return Err((Status::BadRequest, "Username contains invalid characters"));
    }
    if !verify_email_length(signup_request.email) {
        return Err((Status::BadRequest, "Email length is invalid"));
    }
    if !verify_email_format(signup_request.email) {
        return Err((Status::BadRequest, "Invalid email address"));
    }

    match UserRepository::email_exists(signup_request.email)
        .read(db)
        .await
    {
        Ok(true) => {
            warn!("Signup failed: email already in use");
            return Err((Status::Conflict, "Email already in use"));
        }
        Err(err) => {
            error!(error = %err, "Signup failed: could not check email uniqueness");
            return Err((Status::InternalServerError, "Internal server error"));
        }
        _ => (),
    }

    persist_user(
        signup_request.name,
        signup_request.email,
        signup_request.password,
        db,
    )
    .await?;

    Ok(Status::Created)
}

#[cfg(not(feature = "email"))]
fn verify_email_format(email: &str) -> bool {
    // Structural check: one '@', non-empty local and domain parts, domain contains '.'
    let mut parts = email.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn verify_password_length(password: &str) -> bool {
    MIN_PASSWORD_LENGTH as usize <= password.len() && password.len() <= MAX_PASSWORD_LENGTH as usize
}

fn verify_username_length(name: &str) -> bool {
    0 < name.len() && name.len() <= MAX_UTF8_BYTES
}

fn verify_username_content(name: &str) -> bool {
    name.chars().all(|c| !c.is_control())
}

fn verify_email_length(email: &str) -> bool {
    0 < email.len() && email.len() <= MAX_UTF8_BYTES
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::routes;

    use crate::test_harness_setup::{build_test_client, cleanup_user_by_email};

    use super::*;

    #[test]
    fn password_below_min_rejected() {
        assert!(!verify_password_length(
            &"a".repeat(MIN_PASSWORD_LENGTH as usize - 1)
        ));
    }

    #[test]
    fn password_at_min_accepted() {
        assert!(verify_password_length(
            &"a".repeat(MIN_PASSWORD_LENGTH as usize)
        ));
    }

    #[test]
    fn password_at_max_accepted() {
        assert!(verify_password_length(
            &"a".repeat(MAX_PASSWORD_LENGTH as usize)
        ));
    }

    #[test]
    fn password_above_max_rejected() {
        assert!(!verify_password_length(
            &"a".repeat(MAX_PASSWORD_LENGTH as usize + 1)
        ));
    }

    #[cfg(not(feature = "email"))]
    #[test]
    fn valid_email_format_accepted() {
        assert!(verify_email_format("user@example.com"));
        assert!(verify_email_format("user@mail.example.com"));
    }

    #[cfg(not(feature = "email"))]
    #[test]
    fn email_without_at_rejected() {
        assert!(!verify_email_format("notanemail"));
    }

    #[cfg(not(feature = "email"))]
    #[test]
    fn email_without_domain_dot_rejected() {
        assert!(!verify_email_format("user@localhost"));
    }

    #[cfg(not(feature = "email"))]
    #[test]
    fn email_with_empty_local_rejected() {
        assert!(!verify_email_format("@example.com"));
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn signup_returns_201_for_new_user() {
        let client: Client = build_test_client(&routes![signup]).await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test User","email":"newuser@example.com","password":"password123"}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::Created);
        cleanup_user_by_email(
            client.rocket().state::<Pool<MySql>>().unwrap(),
            "newuser@example.com",
        )
        .await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn signup_returns_409_for_duplicate_email() {
        let client = build_test_client(&routes![signup]).await;
        let body =
            r#"{"name":"Test User","email":"duplicate@example.com","password":"password123"}"#;
        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(body)
            .dispatch()
            .await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(body)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::Conflict);
        cleanup_user_by_email(
            client.rocket().state::<Pool<MySql>>().unwrap(),
            "duplicate@example.com",
        )
        .await;
    }

    #[tokio::test]
    #[cfg(feature = "email")]
    #[ignore = "requires database"]
    async fn signup_returns_400_for_invalid_email() {
        let client: Client = build_test_client(&routes![signup]).await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test","email":"notanemail","password":"password123"}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn signup_returns_400_for_short_password() {
        let client: Client = build_test_client(&routes![signup]).await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test","email":"test@example.com","password":"short"}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::BadRequest);
    }
}
