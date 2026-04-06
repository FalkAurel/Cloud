use argon2::{PasswordHash, PasswordVerifier};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket::tokio::task::{self, JoinHandle};
use rocket::{State, post};
use sqlx::{Error, MySql, Pool, Row};
use tracing::{Instrument, error, info, info_span, span, warn};

use crate::data_definitions::{JWT, UserLoginRequest, UserLoginView};
use crate::{ARGON_2, TOKEN_LIFETIME};

const LOGIN_QUERY_STR: &str = r#"
SELECT password AS password_hash, id FROM users WHERE email = ? LIMIT 1;
"#;

// A valid argon2id hash of a random string. Used to run a full verify_password
// when the user does not exist, preventing timing-based user enumeration.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXJhbmRvbXNhbHQ$RoB4RWBSupGkPkOKA7HiYRmFjhSeop6UVKzSFbGMFG4";

#[post("/login", format = "json", data = "<login_request>")]
pub async fn login(
    login_request: Json<UserLoginRequest<'_>>,
    connection: &State<Pool<MySql>>,
    cookies: &CookieJar<'_>,
) -> Result<Status, (Status, &'static str)> {
    let span: tracing::Span = info_span!("login", email = %login_request.email);
    async move {
        let UserLoginView { id, password_hash } =
            get_user_view(connection.inner(), login_request.email)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| UserLoginView {
                    id: -1, // dummy: verify_password will fail against DUMMY_HASH, preventing user enumeration
                    password_hash: DUMMY_HASH.to_owned(),
                });

        let password: String = login_request.into_inner().password.to_owned();

        let span: tracing::Span = tracing::Span::current();
        let join_handle: JoinHandle<Option<String>> = task::spawn_blocking(move || {
            let _guard: span::Entered = span.enter();
            let hash: PasswordHash = match PasswordHash::new(&password_hash) {
                Ok(hash) => hash,
                Err(err) => {
                    error!(error = %err, "Failed to parse password hash");
                    return None;
                }
            };

            match ARGON_2.verify_password(password.as_bytes(), &hash) {
                Ok(_) => match JWT::create(id as u32, TOKEN_LIFETIME) {
                    Ok(token) => {
                        info!(id, "Login successful");
                        Some(token)
                    }
                    Err(err) => {
                        error!(error = %err, id, "Failed to create JWT");
                        None
                    }
                },
                Err(err) => {
                    info!(error = %err, "Login failed");
                    None
                }
            }
        });

        match join_handle.await {
            Ok(Some(token)) => {
                let cookie: Cookie = Cookie::build(("token", token))
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .build();
                cookies.add(cookie);
                Ok(Status::Ok)
            }
            Ok(None) => Err((Status::Unauthorized, "Invalid credentials")),
            Err(err) => {
                error!(error = %err, "Login task panicked");
                Err((Status::InternalServerError, "Internal server error"))
            }
        }
    }
    .instrument(span)
    .await
}

async fn get_user_view(db: &Pool<MySql>, email: &str) -> Result<Option<UserLoginView>, Error> {
    match sqlx::query(LOGIN_QUERY_STR)
        .bind(email)
        .fetch_optional(db)
        .await
    {
        Ok(Some(row)) => {
            let password_hash: String = row.get("password_hash");
            let user_id: i32 = row.get("id");

            Ok(Some(UserLoginView {
                id: user_id,
                password_hash,
            }))
        }
        Ok(None) => {
            info!(email, "Login attempt for unknown email");
            Ok(None)
        }
        Err(err) => {
            warn!(error=%err, "Failed to query the db");
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use sqlx::{MySql, Pool};

    use super::*;
    use crate::data_definitions::init_email_sender;

    async fn build_test_client() -> Client {
        let rocket = rocket::build()
            .mount("/", rocket::routes![login, super::super::signup::signup])
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

    #[tokio::test]
    #[ignore = "requires database and SMTP relay"]
    async fn login_returns_200_after_signup() {
        let client = build_test_client().await;
        let email = "logintest@example.com";
        let password = "password123";

        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"name":"Login Test","email":"{}","password":"{}"}}"#,
                email, password
            ))
            .dispatch()
            .await;

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"email":"{}","password":"{}"}}"#,
                email, password
            ))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Ok);
        cleanup(&client, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database and SMTP relay"]
    async fn login_returns_401_for_wrong_password() {
        let client = build_test_client().await;
        let email = "wrongpass@example.com";

        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"name":"Wrong Pass","email":"{}","password":"correctpassword"}}"#,
                email
            ))
            .dispatch()
            .await;

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"email":"{}","password":"wrongpassword"}}"#,
                email
            ))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
        cleanup(&client, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database and SMTP relay"]
    async fn login_returns_401_for_unknown_email() {
        let client: Client = build_test_client().await;

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email":"nonexistent@example.com","password":"somepassword"}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }
}
