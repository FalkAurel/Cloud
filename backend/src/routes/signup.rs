use argon2::PasswordHasher;
use lettre::Address;
use lettre::transport::smtp::response::{Response, Severity};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::task::JoinHandle;
use rocket::{State, post, tokio};
use sqlx::{Error as SQLError, MySql, Pool, Row};
use std::env;
use std::sync::LazyLock;
use tracing::{error, info, warn};

use crate::ARGON_2;
use crate::data_definitions::{Email, EmailError, EmailSender, UserSignupRequest};

use argon2::password_hash::{SaltString, rand_core::OsRng};

const MIN_PASSWORD_LENGTH: u8 = 8;
const MAX_PASSWORD_LENGTH: u8 = 255;

const INSERT_QUERY_STR: &str = r#"
INSERT INTO users (name, email, password) VALUES (?, ?, ?);
"#;

const CHECK_EMAIL_EXISTS: &str = r#"
SELECT EXISTS(SELECT 1 FROM users WHERE email = ?);
"#;

const SENDER_ADDRESS: LazyLock<Address> = LazyLock::new(|| {
    env::var("MAILER_USER")
        .expect("MAILER_USER must be set")
        .parse()
        .expect("MAILER_USER must be a valid email address")
});
const SIGN_UP_SUBJECT: &str = "Welcome to your own Cloud – You're all set!";
const SIGN_UP_HTML: &str = include_str!("signup_confirmation.html");

#[post("/signup", format = "json", data = "<signup_request>")]
pub async fn signup(
    signup_request: Json<UserSignupRequest<'_>>,
    db: &State<Pool<MySql>>,
    email_sender: &State<EmailSender>,
) -> Result<Status, (Status, &'static str)> {
    let validated_email: Address = match signup_request.email.parse() {
        Ok(email) => email,
        Err(err) => {
            warn!(error = %err, email = signup_request.email, "Signup failed: invalid email address");
            return Err((Status::BadRequest, "Invalid email address"));
        }
    };

    if !verify_password(signup_request.password) {
        return Err((Status::BadRequest, "Password length is invalid"));
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
        Ok(_) => {
            let email: Email = Email::new(SENDER_ADDRESS.clone(), validated_email)
                .set_subject(SIGN_UP_SUBJECT)
                .set_html_content(SIGN_UP_HTML);
            let _ = send_email(email_sender, email).await;
            info!("User signed up successfully");
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

async fn send_email(email_sender: &State<EmailSender>, email: Email<'_>) -> Result<(), EmailError> {
    info!("Sending email");
    let response: Response = email.send(email_sender).await?;

    match response.code().severity {
        Severity::PositiveCompletion | Severity::PositiveIntermediate => {
            info!("Email sent successfully");
        }
        Severity::TransientNegativeCompletion => {
            let error: &str = response.message().next().unwrap_or("Unknown Error");
            warn!(code = ?response.code(), error=%error, "Transient email failure");
        }
        Severity::PermanentNegativeCompletion => {
            warn!(code = ?response.code(), "Permanent email failure");
        }
    }

    Ok(())
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
    use crate::data_definitions::init_email_sender;

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
