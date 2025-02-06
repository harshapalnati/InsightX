mod http_handler;
mod models;
mod config;
mod forwarder;

use axum::Router;
use tokio::{net::TcpListener, sync::mpsc, task};
use std::sync::Arc;
use tracing::info;
use config::Config;
use http_handler::{ingest_log, start_log_processor};
use http_handler::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let config = Config::new();
    let (tx, rx) = mpsc::channel::<models::LogEntry>(10_000);
    let state = Arc::new(AppState { sender: tx });

    // Start the log processor in a background task
    task::spawn(start_log_processor(rx, config.processor_url));

    // Define the HTTP API routes
    let app = Router::new().route("/logs", axum::routing::post(ingest_log)).with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("🚀 Log Collector running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
