#[macro_use]
extern crate rocket;
use rocket::{Config, Rocket, get};
use std::net::{IpAddr, Ipv4Addr};

#[get("/health")]
fn health() -> &'static str {
    "Healthy"
}

#[get("/hi")]
fn hi() -> String {
    "Hello, world!".to_string()
}

#[rocket::main] // Rocket provides a macro to run an async main
async fn main() -> Result<(), rocket::Error> {
    let mut server_config: Config = Config::default();

    server_config.port = 3000;
    server_config.address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    #[cfg(not(debug_assertions))]
    {
        use rocket::config::LogLevel;
        server_config.log_level = LogLevel::Critical;
    }

    let rocket: Rocket<rocket::Ignite> = rocket::build()
        .configure(server_config)
        .mount("/", routes![hi, health])
        .launch()
        .await?;
    dbg!("Rocket launched: {:?}", rocket);
    Ok(())
}
