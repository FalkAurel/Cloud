use argon2::{PasswordHash, PasswordVerifier};
use core::str;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket::tokio::task::{self, JoinHandle};
use rocket::{State, post};
use sqlx::{Error, MySql, Pool, Row};
use std::slice;
use std::sync::LazyLock;
use tracing::{error, info, warn};

use crate::{ARGON_2, TOKEN_LIFETIME};
use crate::data_definitions::{JWT, UserLoginRequest};

const LOGIN_QUERY_STR: &str = r#"
SELECT password AS password_hash, id FROM users WHERE email = ? LIMIT 1;
"#;

struct Password {
    base: usize,
    len: usize,
}

// A valid argon2id hash of a random string. Used to run a full verify_password
// when the user does not exist, preventing timing-based user enumeration.
const DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    String::from(
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXJhbmRvbXNhbHQ$RoB4RWBSupGkPkOKA7HiYRmFjhSeop6UVKzSFbGMFG4",
    )
});

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

    let password: &[u8] = login_request.into_inner().password.as_bytes();
    let password: Password = Password {
        base: password.as_ptr().expose_provenance(),
        len: password.len(),
    };

    let (hash, user_id): (String, Option<u16>) = match result {
        Ok(Some(ref row)) => {
            let hash: String = match row.try_get("password_hash") {
                Ok(hash) => hash,
                Err(err) => {
                    warn!(error = %err, "Failed to retrieve password_hash from row");
                    (*DUMMY_HASH).clone()
                }
            };

            let user_id: Option<u16> = match row.try_get("id") {
                Ok(id) => Some(id),
                Err(err) => {
                    warn!(error = %err, "Failed to retrieve id from row");
                    None
                }
            };
            (hash, user_id)
        }
        Ok(None) => {
            info!("Login failed");
            ((*DUMMY_HASH).clone(), None)
        }
        Err(err) => {
            error!(error = %err, "Database query failed during login");
            ((*DUMMY_HASH).clone(), None)
        }
    };

    let join_handle: JoinHandle<Option<String>> = task::spawn_blocking(move || {
        let hash: PasswordHash = PasswordHash::new(hash.as_str()).unwrap();
        let password: &[u8] =
            unsafe { slice::from_raw_parts(password.base as *const u8, password.len) };

        match ARGON_2.verify_password(password, &hash) {
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
                    warn!("Password verified but user_id missing");
                    None
                }
            }
            Err(err) => {
                info!(error = %err, "Login failed: invalid password");
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
