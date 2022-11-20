use std::net::SocketAddr;
use tracing_subscriber;
use zero2prod::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let address = SocketAddr::from(([0, 0, 0, 0], 8080));
    run(address).await
}
