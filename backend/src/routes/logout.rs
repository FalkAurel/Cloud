use rocket::{post, http::{CookieJar, Status}};

use crate::data_definitions::Auth;

#[post("/logout")]
#[tracing::instrument(skip(_jwt))]
pub async fn logout(_jwt: Auth, cookies: &CookieJar<'_>) -> Result<Status, (Status, &'static str)> {
    cookies.remove("jwt");
    Ok(Status::Ok)
}

#[cfg(test)]
mod tests {
    use rocket::http::{Cookie, Status as HttpStatus};
    use rocket::local::asynchronous::Client;

    use super::*;
    use crate::data_definitions::JWT;
    use crate::TOKEN_LIFETIME;

    async fn build_test_client() -> Client {
        let rocket = rocket::build().mount("/", rocket::routes![logout]);
        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn logout_returns_200_and_clears_cookie() {
        let client = build_test_client().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/logout")
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Ok);

        // Cookie must be removed — either absent or set to empty/expired
        let jwt_cookie = response.cookies().get("jwt");
        assert!(
            jwt_cookie.is_none() || jwt_cookie.unwrap().value().is_empty(),
            "jwt cookie should be cleared after logout"
        );
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn logout_returns_401_without_jwt() {
        let client = build_test_client().await;

        let response = client.post("/logout").dispatch().await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn logout_returns_400_with_invalid_jwt() {
        let client = build_test_client().await;

        let response = client
            .post("/logout")
            .cookie(Cookie::new("jwt", "not.a.valid.jwt"))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }
}