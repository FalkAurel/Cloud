use crate::{
    data_definitions::{Auth, JWT, StandardUserView},
    database::{ReadOnly, Transactional, user_repository::UserRepository},
};
use rocket::{State, delete, http::Status};
use sqlx::{MySql, Pool, Transaction};
use tracing::{error, info, warn};

#[delete("/delete/user/<id>")]
pub async fn delete(
    id: u32,
    auth: Auth,
    db: &State<Pool<MySql>>,
) -> Result<Status, (Status, &'static str)> {
    let jwt: JWT = auth.get_jwt();

    if jwt.user_id == id {
        return delete_user(id, db).await;
    }

    match UserRepository::get_user_info(jwt.user_id).read(db).await {
        Ok(Some(StandardUserView { is_admin: true, .. })) => delete_user(id, db).await,

        Ok(Some(StandardUserView {
            is_admin: false, ..
        })) => {
            info!(user=%jwt.user_id, target_user=%id, "Unauthorized deletion attempt.");
            Err((
                Status::Unauthorized,
                "Unauthorized: you do not have permission to perform this action.",
            ))
        }

        Ok(None) => {
            warn!(user=%jwt.user_id, target_user=%id, "User not found during deletion authorization check.");
            Err((
                Status::Unauthorized,
                "Unauthorized: user account could not be verified.",
            ))
        }

        Err(err) => {
            error!(user=%jwt.user_id, target_user=%id, error=%err, "Database error while checking user permissions.");
            Err((
                Status::InternalServerError,
                "An internal error occurred. Please try again later.",
            ))
        }
    }
}

async fn delete_user(id: u32, db: &Pool<MySql>) -> Result<Status, (Status, &'static str)> {
    // Start transaction
    let mut transaction: Transaction<MySql> = match db.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            error!(
                target_user=%id,
                error=%err,
                "Failed to start database transaction for user deletion."
            );
            return Err((
                Status::InternalServerError,
                "Failed to process request. Please try again later.",
            ));
        }
    };

    let delete_user = UserRepository::delete(id);

    // Execute delete
    if let Err(err) = delete_user.execute(&mut transaction).await {
        error!(target_user=%id, error=%err, "Failed to execute user deletion query.");
        return Err((
            Status::InternalServerError,
            "Failed to delete user. Please try again later.",
        ));
    }

    // Commit transaction
    if let Err(err) = delete_user.commit(transaction).await {
        error!(target_user=%id, error=%err, "Failed to commit transaction for user deletion.");
        return Err((
            Status::InternalServerError,
            "Failed to finalize deletion. Please try again later.",
        ));
    }

    info!(target_user=%id, "User successfully deleted.");

    Ok(Status::Ok)
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Cookie, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::routes;
    use sqlx::{MySql, Pool};

    use crate::TOKEN_LIFETIME;
    use crate::data_definitions::JWT;
    use crate::database::ReadOnly;
    use crate::database::user_repository::UserRepository;
    use crate::routes::{delete_user_request, signup_request};
    use crate::test_harness_setup::{build_test_client, cleanup_user_by_email};

    async fn signup(client: &Client, name: &str, email: &str, password: &str) {
        client
            .post("/signup")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{"name":"{}","email":"{}","password":"{}"}}"#,
                name, email, password
            ))
            .dispatch()
            .await;
    }

    async fn get_id(client: &Client, email: &str) -> u32 {
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        UserRepository::get_login_view(email)
            .read(db)
            .await
            .unwrap()
            .unwrap()
            .id as u32
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn user_can_delete_themselves() {
        let client = build_test_client(&routes![signup_request, delete_user_request]).await;
        let email = "selfdelete@example.com";
        signup(&client, "Self Delete", email, "password123").await;

        let id = get_id(&client, email).await;
        let token = JWT::create(id, TOKEN_LIFETIME).unwrap();

        let response = client
            .delete(format!("/delete/user/{}", id))
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Ok);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_401_without_jwt() {
        let client = build_test_client(&routes![delete_user_request]).await;

        let response = client.delete("/delete/user/1").dispatch().await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn non_admin_cannot_delete_other_user() {
        let client = build_test_client(&routes![signup_request, delete_user_request]).await;
        let attacker_email = "attacker@example.com";
        let victim_email = "victim@example.com";

        signup(&client, "Attacker", attacker_email, "password123").await;
        signup(&client, "Victim", victim_email, "password123").await;

        let attacker_id = get_id(&client, attacker_email).await;
        let victim_id = get_id(&client, victim_email).await;
        let token = JWT::create(attacker_id, TOKEN_LIFETIME).unwrap();

        let response = client
            .delete(format!("/delete/user/{}", victim_id))
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);

        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        cleanup_user_by_email(db, attacker_email).await;
        cleanup_user_by_email(db, victim_email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_401_for_nonexistent_jwt_user() {
        let client = build_test_client(&routes![signup_request, delete_user_request]).await;
        let victim_email = "victim2@example.com";

        signup(&client, "Victim2", victim_email, "password123").await;
        let victim_id = get_id(&client, victim_email).await;

        // JWT references a user that does not exist in the DB
        let token = JWT::create(u32::MAX, TOKEN_LIFETIME).unwrap();

        let response = client
            .delete(format!("/delete/user/{}", victim_id))
            .cookie(Cookie::new("jwt", token))
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);

        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        cleanup_user_by_email(db, victim_email).await;
    }
}
