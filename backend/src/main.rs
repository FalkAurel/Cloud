#[macro_use]
extern crate rocket;
use backend::{init_db, routes::login_request};
use rocket::{Config, Rocket, get};
use std::net::{IpAddr, Ipv4Addr};
use tracing_subscriber::{
    FmtSubscriber,
    fmt::format::{Compact, FmtSpan, Format, JsonFields},
};

#[tracing::instrument]
#[get("/health")]
fn health() -> &'static str {
    "Healthy"
}

#[tracing::instrument]
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

    let subscriber: FmtSubscriber<JsonFields, Format<Compact>> = tracing_subscriber::fmt()
        .json()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    let _rocket: Rocket<rocket::Ignite> = rocket::build()
        .configure(server_config)
        .mount("/", routes![hi, health, login_request])
        .launch()
        .await?;

    Ok(())
}
