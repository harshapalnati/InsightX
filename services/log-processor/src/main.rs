mod http_handler;
mod models;
mod forwarder;
mod config;

use axum::{Router, routing::post};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use http_handler::{ingest_logs, AppState};
use config::Config;

#[tokio::main]
async fn main() {
    // Initialize tracing/logging.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration from the environment.
    let config = Config::new();
    let state = Arc::new(AppState { 
        storage_service_url: config.storage_service_url 
    });

    // Build the Axum application with state.
    let app = Router::new()
        .route("/logs", post(ingest_logs))
        .with_state(state);

    // Define the socket address for binding.
    let addr: SocketAddr = "0.0.0.0:4000".parse().expect("Invalid address");
    info!("🚀 Log Processor running on http://{}", addr);

    // Start the server using axum-server.
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
