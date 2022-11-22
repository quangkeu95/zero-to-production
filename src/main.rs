use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber;
use zero2prod::configuration::get_configuration;
use zero2prod::run;
use std::time::Duration;

const DB_MAX_CONNECTIONS: u32 = 100;
const DB_MAX_LIFETIME: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let configuration = get_configuration()
        .context("Error parsing configuration")
        .unwrap();

    let address = SocketAddr::from(([0, 0, 0, 0], configuration.application_port));

    let connection_string = configuration.database.connection_string();
    let db_connection = PgPoolOptions::new()
        .max_connections(DB_MAX_CONNECTIONS)
        .max_lifetime(DB_MAX_LIFETIME)
        .connect(&connection_string)
        .await
        .context("Failed to connect to database")?;
    run(address, db_connection).await
}
