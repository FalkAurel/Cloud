use argon2::PasswordHasher;
use lettre::Address;
use lettre::transport::smtp::response::{Response, Severity};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::task::JoinHandle;
use rocket::{State, post, tokio};
use sqlx::{Error as SQLError, MySql, Pool, Row};
use std::env;
use std::num::NonZero;
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::ARGON_2;
use crate::data_definitions::{Email, EmailError, EmailSender, UserSignupRequest};

use argon2::password_hash::{SaltString, rand_core::OsRng};

const MIN_PASSWORD_LENGTH: u8 = 8;
const MAX_PASSWORD_LENGTH: u8 = 255;

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

const SIGN_UP_SUBJECT: &str = "Welcome to your own Cloud – You're all set!";
const SIGN_UP_HTML: &str = include_str!("signup_confirmation.html");

#[post("/signup", format = "json", data = "<signup_request>")]
pub async fn signup(
    signup_request: Json<UserSignupRequest<'_>>,
    db: &State<Pool<MySql>>,
    email_sender: &State<EmailSender>,
) -> Result<Status, (Status, &'static str)> {
    if !verify_password(signup_request.password) {
        return Err((Status::BadRequest, "Password length is invalid"));
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
            #[cfg(feature = "email")]
            handle_signup_email(email_sender, validated_email, signup_request.email, db).await?;
            Ok(Status::Created)
        }
        Err(err) => {
            error!(error = %err, "Signup failed: database error");
            Err((Status::InternalServerError, "Internal server error"))
        }
    }
}

fn verify_password(password: &str) -> bool {
    MIN_PASSWORD_LENGTH as usize <= password.len() && password.len() <= MAX_PASSWORD_LENGTH as usize
}

#[cfg(feature = "email")]
async fn handle_signup_email(
    email_sender: &State<EmailSender>,
    email_address: Address,
    raw_email: &str,
    db: &Pool<MySql>,
) -> Result<(), (Status, &'static str)> {
    let email: Email = Email::new(SENDER_ADDRESS.clone(), email_address)
        .set_subject(SIGN_UP_SUBJECT)
        .set_html_content(SIGN_UP_HTML);

    match &*send_email(email_sender, email, RETRIES).await {
        Ok(Ok(_)) => {
            info!("User signed up successfully");
            return Ok(());
        }
        Ok(Err(response)) => {
            warn!(code = ?response.code(), "Email failed to send, but user was created");
            match revert_signup(raw_email, db).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    error!(error=%err, "Failed to revert email");
                    Err((Status::InternalServerError, "Fuck Life"))
                }
            }
        }
        Err(err) => {
            error!(error = %err, "Failed to send email");
            match revert_signup(raw_email, db).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    error!(error=%err, "Failed to revert email");
                    Err((Status::InternalServerError, "Fuck Life"))
                }
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
    email_sender: &State<EmailSender>,
    email: Email<'_>,
    retries: Option<NonZero<u8>>,
) -> Pin<Box<Result<Result<Response, Response>, EmailError>>> {
    info!("Sending email");

    let response: Result<Response, Response> = match email.cheap_clone().send(email_sender).await {
        Ok(response) => match response.code().severity {
            Severity::TransientNegativeCompletion => {
                let error: &str = response.message().next().unwrap_or("Unknown Error");

                if let Some(retries) = retries {
                    info!(remaining_retries = retries.get(), "Retrying email...");
                    sleep(RETRY_WAIT_TIME).await;

                    return Box::pin(send_email(
                        email_sender,
                        email.cheap_clone(),
                        NonZero::new(retries.get() - 1),
                    ))
                    .await;
                }

                warn!(code = ?response.code(), error = %error, "Transient email failure");
                Err(response)
            }

            Severity::PermanentNegativeCompletion => {
                warn!(code = ?response.code(), "Permanent email failure");
                Err(response)
            }

            Severity::PositiveCompletion | Severity::PositiveIntermediate => {
                info!(code = ?response.code(), "Email sent successfully");
                Ok(response)
            }
        },
        Err(err) => {
            error!(error = %err, "Failed to send email");
            return Box::pin(Err(err));
        }
    };

    Box::pin(Ok(response))
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

    #[cfg(feature = "email")]
    async fn build_test_client() -> Client {
        let rocket = rocket::build()
            .mount("/", rocket::routes![signup])
            .manage(crate::init_db().await)
            .manage(init_email_sender().unwrap());
        Client::tracked(rocket).await.unwrap()
    }

    async fn cleanup(client: &Client, email: &str) {
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        sqlx::query("DELETE FROM users WHERE email = ?")
            .bind(email)
            .execute(db)
            .await
            .unwrap();
    }

    #[test]
    fn sender_address_constant_is_valid() {
        let _ = SENDER_ADDRESS.clone();
    }

    #[test]
    fn password_below_min_rejected() {
        assert!(!verify_password(
            &"a".repeat(MIN_PASSWORD_LENGTH as usize - 1)
        ));
    }

    #[test]
    fn password_at_min_accepted() {
        assert!(verify_password(&"a".repeat(MIN_PASSWORD_LENGTH as usize)));
    }

    #[test]
    fn password_at_max_accepted() {
        assert!(verify_password(&"a".repeat(MAX_PASSWORD_LENGTH as usize)));
    }

    #[test]
    fn password_above_max_rejected() {
        assert!(!verify_password(
            &"a".repeat(MAX_PASSWORD_LENGTH as usize + 1)
        ));
    }

    #[tokio::test]
    #[cfg(feature = "email")]
    #[ignore = "requires database and SMTP relay"]
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
    #[cfg(feature = "email")]
    #[ignore = "requires database and SMTP relay"]
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
    #[ignore = "requires database and SMTP relay"]
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
    #[cfg(feature = "email")]
    #[ignore = "requires database and SMTP relay"]
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
