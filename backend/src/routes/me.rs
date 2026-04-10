use rocket::{State, get, http::Status, serde::json::Json};
use sqlx::{MySql, Pool};
use tracing::{error, info, instrument, warn};

use crate::{
    data_definitions::{Auth, StandardUserView},
    database::{ReadOnly, user_repository::UserRepository},
};

#[instrument(skip(jwt, db))]
#[get("/me")]
pub async fn me(
    jwt: Auth,
    db: &State<Pool<MySql>>,
) -> Result<(Status, Json<StandardUserView>), (Status, &'static str)> {
    let user_id: u32 = jwt.0.user_id;

    info!(user_id = user_id, "Fetching current user profile");

    match UserRepository::get_user_info(user_id).read(db).await {
        Ok(Some(user)) => {
            info!(user_id = user_id, "User profile fetched successfully");
            Ok((Status::Ok, Json(user)))
        }
        Ok(None) => {
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
    use rocket::routes;
    use rocket::serde::json;
    use sqlx::{MySql, Pool};

    use crate::TOKEN_LIFETIME;
    use crate::data_definitions::JWT;
    use crate::routes::signup_request;
    use crate::test_harness_setup::{build_test_client, cleanup_user_by_email};

    use super::*;

    async fn get_user_id(client: &Client, email: &str) -> i32 {
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        UserRepository::get_login_view(email)
            .read(db)
            .await
            .unwrap()
            .unwrap()
            .id
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_200_with_valid_jwt() {
        let client: Client = build_test_client(&routes![signup_request, me]).await;
        let email: &str = "metest@example.com";
        let password: &str = "password123";

        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"name":"Me Test","email":"{}","password":"{}"}}"#,
                email, password
            ))
            .dispatch()
            .await;

        let user_id: i32 = get_user_id(&client, email).await;
        let token: String = JWT::create(user_id as u32, TOKEN_LIFETIME).unwrap();

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

        cleanup_user_by_email(client.rocket().state::<Pool<MySql>>().unwrap(), email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_401_without_jwt() {
        let client: Client = build_test_client(&routes![me]).await;

        let response: rocket::local::asynchronous::LocalResponse<'_> =
            client.get("/me").dispatch().await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn me_returns_400_with_invalid_jwt() {
        let client: Client = build_test_client(&routes![me]).await;

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
        let client = build_test_client(&routes![me]).await;
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
