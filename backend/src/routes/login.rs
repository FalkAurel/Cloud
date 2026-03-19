use core::str;
use std::slice;
use std::sync::atomic::{AtomicPtr, Ordering};

use argon2::{PasswordHash, PasswordVerifier};
use rocket::http::Status;
use rocket::tokio::task::{JoinHandle, spawn_blocking};
use rocket::post;
use rocket::serde::json::Json;
use sqlx::{Error, MySql, Pool, Row};

use crate::data_definitions::UserLoginRequest;
use crate::{ARGON_2, DB_POOL};

const LOGIN_QUERY_STR: &str = r#"
SELECT password AS password_hash FROM users WHERE email = ? LIMIT 1;
"#;


/// This is highly dependant on knowing of where the data lives in this context this is safe, but it can not be made public due to hard constraints
struct SendSliceAcrossThreads<T> {
    ptr: AtomicPtr<T>,
    len: usize,
}

impl <T: Sized> SendSliceAcrossThreads<T> {
    pub fn from(value: &[T]) -> Self {
        Self { ptr: AtomicPtr::new(value.as_ptr().cast_mut()), len: value.len() }
    }

    pub fn get(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr.load(Ordering::Relaxed), self.len) }
    }
}


#[post("/login", format = "json", data = "<login_request>")]
pub async fn login(login_request: Json<UserLoginRequest<'_>>) -> (Status, &'static str) {
    let connection: &Pool<MySql> = DB_POOL.get().unwrap();

    let result: Result<Option<sqlx::mysql::MySqlRow>, Error> = sqlx::query(LOGIN_QUERY_STR)
        .bind(login_request.email)
        .fetch_optional(connection)
        .await;

    match result {
        Ok(Some(row)) => {
            let login_password: SendSliceAcrossThreads<u8> = SendSliceAcrossThreads::from(login_request.0.password.as_bytes());

            let task: JoinHandle<(Status, &str)> = spawn_blocking(move || -> (Status, &'static str) {
                let hash_str: &str = match row.try_get("password_hash") {
                    Ok(h) => h,
                    Err(_) => return (Status::InternalServerError, "Internal error"),
                };

                let parsed_hash: PasswordHash = match PasswordHash::new(&hash_str) {
                    Ok(h) => h,
                    Err(err) => return {
                        dbg!(err);
                        (Status::InternalServerError, "Internal error")
                    },
                };
                
                match ARGON_2.verify_password(login_password.get(), &parsed_hash) {
                    Ok(_) => (Status::Ok, "OK"),
                    Err(err) => {
                        dbg!(err);
                        (Status::Unauthorized, "Invalid credentials")
                    }
                }
            });

            let result: (Status, &'static str) = task.await.unwrap();
            result
        }

        Ok(None) => (Status::Unauthorized, "Invalid credentials"),

        Err(err) => {
            dbg!(err);
            (Status::InternalServerError, "Internal error")
        }
    }
}