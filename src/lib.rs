use anyhow::Context;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

pub mod error;

async fn health_check() -> impl IntoResponse {
    "OK"
}

async fn ping() -> impl IntoResponse {
    "pong"
}

pub fn new_router() -> Router {
    let app = Router::new()
        .route("/", get(ping))
        .route("/health_check", get(health_check));
    app
}

pub async fn run(addr: SocketAddr) -> anyhow::Result<()> {
    // build our application with a single route
    let app = new_router();

    println!("Starting HTTP server at {:?}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Error starting HTTP server")
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received, exiting...");
}
