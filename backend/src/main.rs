#[macro_use]
extern crate rocket;
use backend::{RequestTraceSpan, TRACE_LEVEL, init_db, routes::login_request};
use rocket::{Config, Rocket, State, get};
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

    let _rocket: Rocket<rocket::Ignite> = rocket::build()
        .configure(server_config)
        .mount("/", routes![hi, health, login_request])
        .attach(RequestTraceSpan::new())
        .manage(init_db().await)
        .launch()
        .await?;

    Ok(())
}
