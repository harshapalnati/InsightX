mod http_handler;
mod file_ingestion;
mod tcp_ingestion;
mod udp_ingestion;
mod forwarder;
mod models;
mod config;
mod syslog_ingestion;
mod docker_ingestion;
mod awscloudwatch;

// mod kafka_ingestion;

use axum::Router;
use tokio::{net::TcpListener, sync::mpsc, task};
use std::{sync::Arc, path::PathBuf};
use tracing::info;
use config::Config;
use http_handler::{ingest_log, start_log_processor};
use http_handler::AppState;
use file_ingestion::{watch_log_files, FileIngestionConfig};
use tcp_ingestion::start_tcp_server;
use udp_ingestion::start_udp_listener;
use syslog_ingestion::start_syslog_listener;
 use docker_ingestion::{start_docker_log_ingestion, DockerIngestionConfig};
 use awscloudwatch::{start_aws_log_ingestion, AWSCloudWatchConfig};

// use kafka_ingestion::start_kafka_listener;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    let config = Config::new();
    let (tx, rx) = mpsc::channel::<models::LogEntry>(10_000);
    let state = Arc::new(AppState { sender: tx.clone() });

   

     // 🔹 Start AWS CloudWatch Log Ingestion
     let aws_config = Arc::new(AWSCloudWatchConfig {
        log_group_name: "your-log-group-name".to_string(), // Change this
        log_stream_name: Some("your-log-stream-name".to_string()), // Change this
    });

    task::spawn(start_aws_log_ingestion(aws_config, tx.clone()));

    let docker_config = Arc::new(DockerIngestionConfig {
        container_name: "test-container".to_string(),
    });
    
    task::spawn(start_docker_log_ingestion(docker_config, tx.clone()));
    // Start UDP log ingestion
    let udp_port = "0.0.0.0:5051";
    task::spawn(start_udp_listener(udp_port, tx.clone()));

    // Start Syslog ingestion (UDP port 514 is default for Syslog)
    let syslog_port = "0.0.0.0:514";
    task::spawn(start_syslog_listener(syslog_port, tx.clone()));


    // 🔹 Start File Log Ingestion
    let file_config = Arc::new(FileIngestionConfig {
        log_directory: PathBuf::from("./logs"),
    });
    task::spawn(watch_log_files(file_config, tx.clone()));

    // 🔹 Start TCP Log Server
    task::spawn(start_tcp_server("0.0.0.0:5050", tx.clone()));

    // 🔹 Start Log Processor
    task::spawn(start_log_processor(rx, config.processor_url));

    // 🔹 Define HTTP API routes
    let app = Router::new().route("/logs", axum::routing::post(ingest_log)).with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("🚀 Log Collector running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
