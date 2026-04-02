use argon2::{PasswordHash, PasswordVerifier};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket::tokio::task::{self, JoinHandle};
use rocket::{State, post};
use sqlx::{Error, MySql, Pool, Row};
use tracing::{error, info, warn};

use crate::{ARGON_2, TOKEN_LIFETIME};
use crate::data_definitions::{JWT, UserLoginRequest};

const LOGIN_QUERY_STR: &str = r#"
SELECT password AS password_hash, id FROM users WHERE email = ? LIMIT 1;
"#;

// A valid argon2id hash of a random string. Used to run a full verify_password
// when the user does not exist, preventing timing-based user enumeration.
const DUMMY_HASH: &str = 
"$argon2id$v=19$m=19456,t=2,p=1$c29tZXJhbmRvbXNhbHQ$RoB4RWBSupGkPkOKA7HiYRmFjhSeop6UVKzSFbGMFG4";

#[post("/login", format = "json", data = "<login_request>")]
pub async fn login(
    login_request: Json<UserLoginRequest<'_>>,
    connection: &State<Pool<MySql>>,
    cookies: &CookieJar<'_>,
) -> Result<Status, (Status, &'static str)> {
    let result: Result<Option<sqlx::mysql::MySqlRow>, Error> = sqlx::query(LOGIN_QUERY_STR)
        .bind(login_request.email)
        .fetch_optional(connection.inner())
        .await;

    let password: String = login_request.into_inner().password.to_owned();

    let (hash, user_id): (Option<String>, Option<u32>) = match result {
        Ok(Some(ref row)) => {
            let hash: Option<String> = match row.try_get("password_hash") {
                Ok(hash) => Some(hash),
                Err(err) => {
                    warn!(error = %err, "Failed to retrieve password_hash from row");
                    None
                }
            };

            let user_id: Option<u32> = match row.try_get("id") {
                Ok(id) => Some(id),
                Err(err) => {
                    warn!(error = %err, "Failed to retrieve id from row");
                    None
                }
            };
            (hash, user_id)
        }
        Ok(None) => {
            warn!("Login failed");
            (None, None)
        }
        Err(err) => {
            error!(error = %err, "Login failed");
            (None, None)
        }
    };

    let join_handle: JoinHandle<Option<String>> = task::spawn_blocking(move || {
        let hash: PasswordHash = match PasswordHash::new(hash.as_deref().unwrap_or(DUMMY_HASH)) {
            Ok(hash) => hash,
            Err(err) => {
                error!(error = %err, "Failed to parse password hash");
                return None;
            }
        };

        match ARGON_2.verify_password(password.as_bytes(), &hash) {
            Ok(_) => {
                if let Some(user_id) = user_id {
                    match JWT::create(user_id, TOKEN_LIFETIME) {
                        Ok(token) => {
                            info!(user_id, "Login successful");
                            Some(token)
                        }
                        Err(err) => {
                            error!(error = %err, user_id, "Failed to create JWT");
                            None
                        }
                    }
                } else {
                    warn!("Login failed");
                    None
                }
            }
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
