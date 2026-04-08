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
    type Error;

    fn read(
        &self,
        pool: &Pool<MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send;
}

pub(crate) trait Transactional {
    type Success;
    type Error;

    fn execute<'t>(
        &self,
        tx: &'t mut Transaction<'_, MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send;

    fn commit(
        self,
        tx: Transaction<'_, MySql>,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        Self: Sized,
    {
        tx.commit()
    }

    fn rollback(tx: Transaction<'_, MySql>) -> impl Future<Output = Result<(), sqlx::Error>> + Send
    where
        Self: Sized,
    {
        tx.rollback()
    }
}

pub(crate) mod user_repository;
