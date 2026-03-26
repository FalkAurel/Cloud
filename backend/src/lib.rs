use std::{
    env,
    sync::{LazyLock, OnceLock},
};

use argon2::Argon2;
use sqlx::{MySql, Pool, mysql::MySqlConnectOptions};
use tracing::Level;
use tracing_subscriber::EnvFilter;

pub mod data_definitions;
pub mod routes;

#[cfg(debug_assertions)]
mod http_span;
#[cfg(debug_assertions)]
pub use http_span::{HttpSpan, RequestTraceSpan};


pub(crate) static DB_POOL: OnceLock<Pool<MySql>> = OnceLock::new();
pub(crate) static ARGON_2: LazyLock<Argon2> = LazyLock::new(|| Argon2::default());

pub static TRACE_LEVEL: LazyLock<Level> = LazyLock::new(|| {
    let log_level: Level = EnvFilter::from_default_env()
        .max_level_hint()
        .and_then(|hint| hint.into_level())
        .unwrap_or(Level::INFO);

    tracing::info!("Setting log level to {}", log_level);
    log_level
});

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
