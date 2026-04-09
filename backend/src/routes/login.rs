use argon2::{PasswordHash, PasswordVerifier};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket::tokio::task::{self, JoinHandle};
use rocket::{State, post};
use sqlx::{MySql, Pool};
use tracing::{Instrument, Span, error, info, info_span, span::Entered, warn};

use crate::data_definitions::{JWT, UserLoginRequest, UserLoginView};
use crate::database::ReadOnly;
use crate::database::user_repository::UserRepository;
use crate::{ARGON_2, TOKEN_LIFETIME};

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
            UserRepository::get_login_view(login_request.email)
                .read(connection)
                .await
                .unwrap_or_else(|err| {
                    warn!(error = %err, "DB error fetching login view, falling back to dummy hash");
                    None
                })
                .unwrap_or_else(|| UserLoginView {
                    id: -1,
                    password_hash: DUMMY_HASH.to_owned(),
                });

        let password: String = login_request.into_inner().password.to_owned();

        let blocking_span: Span = tracing::Span::current();
        let join_handle: JoinHandle<Option<String>> = task::spawn_blocking(move || {
            let _guard: Entered = blocking_span.enter();
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
                let cookie: Cookie = Cookie::build(("jwt", token))
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

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::routes;
    use sqlx::{MySql, Pool};

    use crate::routes::{login_request, signup_request};
    use crate::test_harness_setup::build_test_client;

    async fn cleanup(client: &Client, email: &str) {
        use crate::database::Transactional;
        use crate::database::user_repository::UserRepository;
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        let mut tx = db.begin().await.unwrap();
        let delete = UserRepository::delete(email);
        delete.execute(&mut tx).await.unwrap();
        delete.commit(tx).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn login_returns_200_after_signup() {
        let client: Client = build_test_client(&routes![signup_request, login_request]).await;
        let email: &str = "logintest@example.com";
        let password: &str = "password123";

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
    #[ignore = "requires database"]
    async fn login_returns_401_for_wrong_password() {
        let client = build_test_client(&routes![signup_request, login_request]).await;
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
    #[ignore = "requires database"]
    async fn login_returns_401_for_unknown_email() {
        let client: Client = build_test_client(&routes![login_request]).await;

        let response = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email":"nonexistent@example.com","password":"somepassword"}"#)
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }
}
