use std::{
    env,
    sync::{LazyLock, OnceLock},
};

use argon2::Argon2;
use sqlx::{MySql, Pool, mysql::MySqlConnectOptions};

pub mod data_definitions;
pub mod routes;

pub(crate) static DB_POOL: OnceLock<Pool<MySql>> = OnceLock::new();
pub(crate) static ARGON_2: LazyLock<Argon2> = LazyLock::new(|| Argon2::default());

pub async fn init_db() {
    let user: String = env::var("MARIADB_USER").expect("Provide a USER");
    let password: String = env::var("MARIADB_PASSWORD").expect("Provide a Password");
    let database: String = env::var("MARIADB_DATABASE").expect("Provide a database");

    let connection_pool: Pool<MySql> = Pool::connect_lazy_with(
        MySqlConnectOptions::new()
            .host("db")
            .username(&user)
            .password(&password)
            .database(&database),
    );

    DB_POOL
        .set(connection_pool)
        .expect("DB_POOL was already initialized");
}
