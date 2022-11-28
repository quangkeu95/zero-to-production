use anyhow::Context;
use dotenvy::dotenv;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::info;
use tracing::log::LevelFilter;
use zero2prod::configuration::get_configuration;
use zero2prod::telemetry::parse_log_level;
use zero2prod::{run, telemetry};

const DB_MAX_CONNECTIONS: u32 = 100;
const DB_MAX_LIFETIME: Duration = Duration::from_secs(3);
const DB_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let crate_log_level = parse_log_level(std::env::var("APP_LOG_LEVEL"), Some(LevelFilter::Info));
    let http_log_level = parse_log_level(std::env::var("HTTP_LOG_LEVEL"), None);
    let tracing_options = telemetry::TracingOptionsBuilder::default()
        .crate_level(crate_log_level)
        .tower_http_level(http_log_level)
        .build()?;

    // initialize tracing
    telemetry::init_tracing("zero2prod".into(), tracing_options);

    let configuration = get_configuration()
        .context("Error parsing configuration")
        .unwrap();

    let address = SocketAddr::from(([0, 0, 0, 0], configuration.application.port));

    let db_connection = PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .max_lifetime(DB_MAX_LIFETIME)
        .acquire_timeout(DB_ACQUIRE_TIMEOUT)
        .connect_with(configuration.database.with_db())
        .await
        .context("Failed to connect to database")?;
    run(address, db_connection).await
}
