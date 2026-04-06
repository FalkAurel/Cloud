#[macro_use]
extern crate rocket;
use backend::{
    TRACE_LEVEL,
    data_definitions::init_email_sender,
    init_db,
    routes::{login_request, signup_request},
};

use rocket::{Config, Rocket, get};
use rocket_cors::{AllowedOrigins, CorsOptions};
use std::net::{IpAddr, Ipv4Addr};
use tracing_subscriber::fmt::{format::FmtSpan, writer::MakeWriterExt};

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

#[cfg(not(feature = "export_binding"))]
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    use std::path::PathBuf;

    let mut server_config: Config = Config::default();

    server_config.port = 3000;
    server_config.address = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));

    #[cfg(not(debug_assertions))]
    {
        use rocket::config::LogLevel;
        server_config.log_level = LogLevel::Critical;
    }

    #[cfg(debug_assertions)]
    fastrace::set_reporter(
        fastrace::collector::ConsoleReporter,
        fastrace::collector::Config::default(),
    );

    if cfg!(debug_assertions) {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stdout.with_min_level(*TRACE_LEVEL))
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");
    } else {
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(std::io::stdout.with_min_level(*TRACE_LEVEL))
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");
    };

    tracing::info!(level = %*TRACE_LEVEL, "Tracing initialized");

    let cors: rocket_cors::Cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&["http://localhost:5173"]))
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to build CORS");

    let _rocket: Rocket<rocket::Ignite> = rocket::build()
        .configure(server_config)
        .mount("/", routes![hi, health, login_request, signup_request])
        .manage(init_db().await)
        .manage(init_email_sender().unwrap())
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}

#[cfg(feature = "export_binding")]
fn main() {
    use std::path::PathBuf;

    use backend::data_definitions::*;
    use ts_rs::{Config, TS};

    let config: Config = Config::new().with_out_dir(
        ["..", "frontend", "vue-project", "src", "types", "bindings"]
            .iter()
            .collect::<PathBuf>(),
    );

    StandardUserView::export_all(&config).unwrap();
    UserLoginRequest::export_all(&config).unwrap();
    UserSignupRequest::export_all(&config).unwrap();
}
