#[macro_use]
extern crate rocket;
use rocket::{Config, Rocket, get, tokio::sync::OnceCell};
use sqlx::{MySql, Pool, mysql::MySqlConnectOptions};
use std::{env, net::{IpAddr, Ipv4Addr}};

static DB_POOL: OnceCell<Pool<MySql>> = OnceCell::const_new();

#[get("/health")]
fn health() -> &'static str {
    "Healthy"
}

#[get("/hi")]
fn hi() -> String {
    "Hello, world!".to_string()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let mut server_config: Config = Config::default();

    server_config.port = 3000;
    server_config.address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    #[cfg(not(debug_assertions))]
    {
        use rocket::config::LogLevel;
        server_config.log_level = LogLevel::Critical;
    }
    
    init_db().await;
    let rocket: Rocket<rocket::Ignite> = rocket::build()
        .configure(server_config)
        .mount("/", routes![hi, health])
        .launch()
        .await?;


    dbg!("Rocket launched: {:?}", rocket);
    Ok(())
}


async fn init_db() {
    let user: String = env::var("MARIADB_USER").expect("Provide a USER");
    let password: String = env::var("MARIADB_PASSWORD").expect("Provide a Password");
    let database: String = env::var("MARIADB_DATABASE").expect("Provide a database");

    let connection_pool: Pool<MySql> = Pool::connect_lazy_with(MySqlConnectOptions::new()
        .host("db")
        .username(&user)
        .password(&password)
        .database(&database));

    DB_POOL.set(connection_pool).expect("DB_POOL was already initialized");
}
