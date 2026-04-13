use std::{sync::LazyLock, time::Duration};

use argon2::Argon2;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "export_binding")]
pub mod data_definitions;
#[cfg(not(feature = "export_binding"))]
pub(crate) mod data_definitions;
pub(crate) mod database;
pub(crate) mod object_storage;
pub mod routes;
pub use database::init_db;

#[cfg(feature = "email")]
pub use data_definitions::{init_email_sender, EmailSenderConfig};

pub(crate) static ARGON_2: LazyLock<Argon2> = LazyLock::new(|| Argon2::default());
pub(crate) const TOKEN_LIFETIME: Duration = Duration::from_mins(10);

pub static TRACE_LEVEL: LazyLock<Level> = LazyLock::new(|| {
    let log_level: Level = EnvFilter::from_default_env()
        .max_level_hint()
        .and_then(|hint| hint.into_level())
        .unwrap_or(Level::INFO);

    log_level
});

#[cfg(test)]
pub(crate) mod test_harness_setup {
    use rocket::{Route, local::asynchronous::Client};
    use sqlx::{MySql, Pool};

    pub(crate) async fn build_test_client(routes: &[Route]) -> Client {
        #[cfg(feature = "email")]
        {
            use rocket::Rocket;

            use crate::{data_definitions::init_email_sender, init_db};

            let config = init_email_sender().unwrap();
            let rocket = Rocket::build()
                .mount("/", routes)
                .manage(init_db().await)
                .manage(config.sender)
                .manage(config.sender_address);
            Client::tracked(rocket).await.unwrap()
        }

        #[cfg(not(feature = "email"))]
        {
            use crate::init_db;
            use rocket::Rocket;

            let rocket = Rocket::build().mount("/", routes).manage(init_db().await);
            Client::tracked(rocket).await.unwrap()
        }
    }

    pub(crate) async fn cleanup_user_by_email(pool: &Pool<MySql>, email: &str) {
        use crate::database::{ReadOnly, Transactional, user_repository::UserRepository};

        let id: i32 = UserRepository::get_login_view(email)
            .read(pool)
            .await
            .unwrap()
            .unwrap()
            .id;
        let mut tx: sqlx::Transaction<'_, MySql> = pool.begin().await.unwrap();
        let delete = UserRepository::delete(id);
        delete.execute(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
    }
}
