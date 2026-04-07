use rocket::{State, get, http::Status, serde::json::Json};
use sqlx::{Error, MySql, Pool, Row};
use tracing::{error, info, warn};

use crate::data_definitions::{Auth, FixedSizedStr, StandardUserView};

const GET_USER_INFO: &str = r#"
SELECT name, email, is_admin FROM users WHERE id = ? LIMIT 1;
"#;

#[get("/me")]
pub async fn me(
    jwt: Auth,
    db_connection: &State<Pool<MySql>>,
) -> Result<(Status, Json<StandardUserView>), (Status, &'static str)> {
    let user_id: u32 = jwt.0.user_id;

    info!(user_id = user_id, "Fetching current user profile");

    match sqlx::query(GET_USER_INFO)
        .bind(user_id as i32)
        .fetch_one(db_connection.inner())
        .await
    {
        Ok(row) => {
            let name: FixedSizedStr<160> = FixedSizedStr::new_from_str(row.get::<&str, usize>(0))
                .expect("DB contains invalid name that passed signup validation");
            let email: FixedSizedStr<160> = FixedSizedStr::new_from_str(row.get::<&str, usize>(1))
                .expect("DB contains invalid email that passed signup validation");
            let is_admin: bool = row.get(2);

            info!(user_id = user_id, "User profile fetched successfully");

            Ok((
                Status::Ok,
                Json(StandardUserView {
                    name,
                    email,
                    is_admin,
                }),
            ))
        }

        Err(Error::RowNotFound) => {
            warn!(user_id = user_id, "User ID from JWT not found in database");
            Err((Status::Unauthorized, "User not found"))
        }

        Err(e) => {
            error!(user_id = user_id, error = %e, "Database error while fetching user");
            Err((Status::InternalServerError, "Internal server error"))
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Cookie, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json;
    use sqlx::{MySql, Pool};

    use crate::TOKEN_LIFETIME;
    use crate::data_definitions::JWT;

    #[cfg(feature = "email")]
    use crate::data_definitions::init_email_sender;

    use super::*;

    async fn build_test_client() -> Client {
        let mut rocket = rocket::build()
            .mount(
                "/",
                rocket::routes![me, super::super::signup::signup, super::super::login::login],
            )
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

    async fn get_user_id(client: &Client, email: &str) -> i32 {
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        sqlx::query("SELECT id FROM users WHERE email = ? LIMIT 1")
            .bind(email)
            .fetch_one(db)
            .await
            .unwrap()
            .get::<i32, usize>(0)
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_200_with_valid_jwt() {
        let client = build_test_client().await;
        let email = "metest@example.com";
        let password = "password123";

        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"name":"Me Test","email":"{}","password":"{}"}}"#,
                email, password
            ))
            .dispatch()
            .await;

        let user_id = get_user_id(&client, email).await;
        let token = JWT::create(user_id as u32, TOKEN_LIFETIME).unwrap();

        let response = client
            .get("/me")
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Ok);

        let body: json::Value = json::from_str(&response.into_string().await.unwrap()).unwrap();
        assert_eq!(body["email"], email);
        assert_eq!(body["name"], "Me Test");
        assert_eq!(body["is_admin"], false);

        cleanup(&client, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_401_without_jwt() {
        let client = build_test_client().await;

        let response = client.get("/me").dispatch().await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_400_with_invalid_jwt() {
        let client = build_test_client().await;

        let response = client
            .get("/me")
            .cookie(Cookie::new("jwt", "not.a.valid.jwt"))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_401_for_nonexistent_user() {
        let client = build_test_client().await;
        // Use a user_id that does not exist in the database
        let token = JWT::create(u32::MAX, TOKEN_LIFETIME).unwrap();

        let response = client
            .get("/me")
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }
}
