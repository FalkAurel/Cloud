use argon2::PasswordHasher;
#[cfg(feature = "email")]
use lettre::Address;
#[cfg(feature = "email")]
use lettre::transport::smtp::response::{Response, Severity};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::task::JoinHandle;
use rocket::{State, post, tokio};
use sqlx::{Error as SQLError, MySql, Pool, Row};
#[cfg(feature = "email")]
use std::env;
#[cfg(feature = "email")]
use std::num::NonZero;
#[cfg(feature = "email")]
use std::sync::LazyLock;
#[cfg(feature = "email")]
use std::time::Duration;
#[cfg(feature = "email")]
use tokio::time::sleep;
use tracing::{error, warn};
#[cfg(feature = "email")]
use tracing::info;

use crate::ARGON_2;
use crate::data_definitions::{MAX_UTF8_BYTES, UserSignupRequest};
#[cfg(feature = "email")]
use crate::data_definitions::{Email, EmailError, EmailSender};

use argon2::password_hash::{SaltString, rand_core::OsRng};

const MIN_PASSWORD_LENGTH: u8 = 8;
const MAX_PASSWORD_LENGTH: u8 = MAX_UTF8_BYTES as u8;

#[cfg(feature = "email")]
const DELETE_USER_QUERY_STR: &str = r#"
DELETE LOW_PRIORITY FROM users WHERE email = ?;
"#;

const INSERT_QUERY_STR: &str = r#"
INSERT INTO users (name, email, password) VALUES (?, ?, ?);
"#;

const CHECK_EMAIL_EXISTS: &str = r#"
SELECT EXISTS(SELECT 1 FROM users WHERE email = ?);
"#;

#[cfg(feature = "email")]
const SENDER_ADDRESS: LazyLock<Address> = LazyLock::new(|| {
    env::var("MAILER_USER")
        .expect("MAILER_USER must be set")
        .parse()
        .expect("MAILER_USER must be a valid email address")
});

#[cfg(feature = "email")]
const RETRY_WAIT_TIME: Duration = Duration::from_secs(30);

#[cfg(feature = "email")]
const RETRIES: Option<NonZero<u8>> = NonZero::new(3);

#[cfg(feature = "email")]
const SIGN_UP_SUBJECT: &str = "Welcome to your own Cloud – You're all set!";
#[cfg(feature = "email")]
const SIGN_UP_HTML: &str = include_str!("signup_confirmation.html");

#[cfg(feature = "email")]
#[post("/signup", format = "json", data = "<signup_request>")]
pub async fn signup(
    signup_request: Json<UserSignupRequest<'_>>,
    db: &State<Pool<MySql>>,
    email_sender: &State<EmailSender>,
) -> Result<Status, (Status, &'static str)> {
    if !verify_password_length(signup_request.password) {
        return Err((Status::BadRequest, "Password length is invalid"));
    }

    if !verify_username_length(signup_request.name) {
        return Err((Status::BadRequest, "Username length is invalid"));
    }

    if !verify_email_length(signup_request.email) {
        return Err((Status::BadRequest, "Email length is invalid"));
    }

    let validated_email: Address = match signup_request.email.parse() {
        Ok(email) => email,
        Err(err) => {
            warn!(error = %err, email = signup_request.email, "Signup failed: invalid email address");
            return Err((Status::BadRequest, "Invalid email address"));
        }
    };

    match email_exists(db, signup_request.email).await {
        Ok(true) => {
            warn!(
                email = signup_request.email,
                "Signup failed: email already in use"
            );
            return Err((Status::Conflict, "Email already in use"));
        }
        Err(err) => {
            error!(error = %err, "Signup failed: could not check email uniqueness");
            return Err((Status::InternalServerError, "Internal server error"));
        }
        _ => (),
    }

    let salt_string: SaltString = SaltString::generate(&mut OsRng);
    let password: String = signup_request.password.to_owned();
    let handle: JoinHandle<Result<String, (Status, &'static str)>> =
        tokio::task::spawn_blocking(move || {
            match ARGON_2.hash_password(password.as_bytes(), &salt_string) {
                Ok(hash) => Ok(hash.to_string()),
                Err(err) => {
                    error!(error = %err, "Failed to hash password");
                    return Err((Status::InternalServerError, "Failed to hash password"));
                }
            }
        });

    let hashed_password: String = handle.await.or_else(|err| {
        warn!(error = %err, "Hashing Task failed");
        Err((Status::InternalServerError, "Internal server error"))
    })??;

    match sqlx::query(INSERT_QUERY_STR)
        .bind(signup_request.name)
        .bind(signup_request.email)
        .bind(&hashed_password)
        .execute(db.inner())
        .await
    {
        Ok(_) => {
            let sender: lettre::AsyncSmtpTransport<lettre::Tokio1Executor> =
                email_sender.inner().clone();
            let pool: Pool<MySql> = db.inner().clone();
            let raw_email: String = signup_request.email.to_owned();
            tokio::spawn(handle_signup_email(
                sender,
                validated_email,
                raw_email,
                pool,
            ));
            Ok(Status::Created)
        }
        Err(err) => {
            error!(error = %err, "Signup failed: database error");
            Err((Status::InternalServerError, "Internal server error"))
        }
    }
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

    if !verify_email_length(signup_request.email) {
        return Err((Status::BadRequest, "Email length is invalid"));
    }

    match email_exists(db, signup_request.email).await {
        Ok(true) => {
            warn!(
                email = signup_request.email,
                "Signup failed: email already in use"
            );
            return Err((Status::Conflict, "Email already in use"));
        }
        Err(err) => {
            error!(error = %err, "Signup failed: could not check email uniqueness");
            return Err((Status::InternalServerError, "Internal server error"));
        }
        _ => (),
    }

    let salt_string: SaltString = SaltString::generate(&mut OsRng);
    let password: String = signup_request.password.to_owned();
    let handle: JoinHandle<Result<String, (Status, &'static str)>> =
        tokio::task::spawn_blocking(move || {
            match ARGON_2.hash_password(password.as_bytes(), &salt_string) {
                Ok(hash) => Ok(hash.to_string()),
                Err(err) => {
                    error!(error = %err, "Failed to hash password");
                    return Err((Status::InternalServerError, "Failed to hash password"));
                }
            }
        });

    let hashed_password: String = handle.await.or_else(|err| {
        warn!(error = %err, "Hashing Task failed");
        Err((Status::InternalServerError, "Internal server error"))
    })??;

    match sqlx::query(INSERT_QUERY_STR)
        .bind(signup_request.name)
        .bind(signup_request.email)
        .bind(&hashed_password)
        .execute(db.inner())
        .await
    {
        Ok(_) => Ok(Status::Created),
        Err(err) => {
            error!(error = %err, "Signup failed: database error");
            Err((Status::InternalServerError, "Internal server error"))
        }
    }
}

fn verify_password_length(password: &str) -> bool {
    MIN_PASSWORD_LENGTH as usize <= password.len() && password.len() <= MAX_PASSWORD_LENGTH as usize
}

fn verify_username_length(name: &str) -> bool {
    0 < name.len() && name.len() <= MAX_UTF8_BYTES
}

fn verify_email_length(email: &str) -> bool {
    0 < email.len() && email.len() <= MAX_UTF8_BYTES
}

#[cfg(feature = "email")]
async fn handle_signup_email(
    email_sender: EmailSender,
    email_address: Address,
    raw_email: String,
    db: Pool<MySql>,
) {
    let email: Email = Email::new(SENDER_ADDRESS.clone(), email_address)
        .set_subject(SIGN_UP_SUBJECT)
        .set_html_content(SIGN_UP_HTML);

    match send_email(&email_sender, email, RETRIES).await {
        Ok(Ok(_)) => {
            info!("User signed up successfully");
        }
        Ok(Err(response)) => {
            warn!(code = ?response.code(), "Email failed to send, but user was created");
            if let Err(err) = revert_signup(&raw_email, &db).await {
                error!(error=%err, "Failed to revert signup after email failure");
            }
        }
        Err(err) => {
            error!(error = %err, "Failed to send email");
            if let Err(err) = revert_signup(&raw_email, &db).await {
                error!(error=%err, "Failed to revert signup after email failure");
            }
        }
    }
}

#[cfg(feature = "email")]
async fn revert_signup(email: &str, db: &Pool<MySql>) -> Result<(), SQLError> {
    sqlx::query(DELETE_USER_QUERY_STR)
        .bind(email)
        .execute(db)
        .await?;

    Ok(())
}

#[cfg(feature = "email")]
async fn send_email(
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

async fn email_exists(db: &Pool<MySql>, email: &str) -> Result<bool, SQLError> {
    Ok(sqlx::query(CHECK_EMAIL_EXISTS)
        .bind(email)
        .fetch_one(db)
        .await?
        .get::<bool, usize>(0))
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status as HttpStatus};
    use rocket::local::asynchronous::Client;

    use super::*;

    #[cfg(feature = "email")]
    use crate::data_definitions::init_email_sender;

    async fn build_test_client() -> Client {
        let mut rocket = rocket::build()
            .mount("/", rocket::routes![signup])
            .manage(crate::init_db().await);

        #[cfg(feature = "email")]
        {
            rocket = rocket.manage(init_email_sender().unwrap());
            return Client::tracked(rocket).await.unwrap();
        }

        #[cfg(not(feature = "email"))]
        return Client::tracked(rocket).await.unwrap()
    }

    async fn cleanup(client: &Client, email: &str) {
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        sqlx::query("DELETE FROM users WHERE email = ?")
            .bind(email)
            .execute(db)
            .await
            .unwrap();
    }

    #[cfg(feature = "email")]
    #[test]
    fn sender_address_constant_is_valid() {
        let _ = *SENDER_ADDRESS;
    }

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

    #[tokio::test]
    #[ignore = "requires database"]
    async fn signup_returns_201_for_new_user() {
        let client = build_test_client().await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test User","email":"newuser@example.com","password":"password123"}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::Created);
        cleanup(&client, "newuser@example.com").await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn signup_returns_409_for_duplicate_email() {
        let client = build_test_client().await;
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
        cleanup(&client, "duplicate@example.com").await;
    }

    #[tokio::test]
    #[cfg(feature = "email")]
    #[ignore = "requires database"]
    async fn signup_returns_400_for_invalid_email() {
        let client = build_test_client().await;
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
        let client = build_test_client().await;
        let response = client
            .post("/signup")
            .header(ContentType::JSON)
            .body(r#"{"name":"Test","email":"test@example.com","password":"short"}"#)
            .dispatch()
            .await;
        assert_eq!(response.status(), HttpStatus::BadRequest);
    }
}
