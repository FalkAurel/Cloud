use sqlx::{MySql, Pool, Transaction, mysql::MySqlConnectOptions};
use std::env;

pub async fn init_db() -> Pool<MySql> {
    let user: String = env::var("MARIADB_USER").expect("Provide a USER");
    let password: String = env::var("MARIADB_PASSWORD").expect("Provide a Password");
    let database: String = env::var("MARIADB_DATABASE").expect("Provide a database");
    let host: String = env::var("MARIADB_HOST").unwrap_or_else(|_| "db".to_string());

    let connection_pool: Pool<MySql> = Pool::connect_lazy_with(
        MySqlConnectOptions::new()
            .host(&host)
            .username(&user)
            .password(&password)
            .database(&database),
    );

    connection_pool
}

pub(crate) trait ReadOnly {
    type Success;
    type Error: std::fmt::Debug + std::fmt::Display + Send;

    fn read(
        &self,
        pool: &Pool<MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send;
}

pub(crate) trait Transactional {
    type Success: Send;
    type Error: std::fmt::Debug + std::fmt::Display + Send;

    fn execute<'t>(
        &self,
        tx: &'t mut Transaction<'_, MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send;
}

pub(crate) mod user_repository;
pub(crate) mod virtual_filesystem;
