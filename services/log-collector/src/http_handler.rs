use axum::{extract::State, http::StatusCode};
use tokio::sync::mpsc;
use tracing::{info, error};
use std::{sync::Arc, str};
use crate::{forwarder::send_logs, models::LogEntry};
use bytes::Bytes;
use serde_json::Value;

#[derive(Clone)]
pub struct AppState {
    pub sender: mpsc::Sender<LogEntry>,
}

// ✅ Optimized Log Ingestion Handler
pub async fn ingest_log(
    State(state): State<Arc<AppState>>,
    body: Bytes // Use Zero-Copy `Bytes` for performance
) -> StatusCode {

    if let Ok(body_str) = str::from_utf8(&body) {
        let json: Value = match serde_json::from_str(body_str) {
            Ok(data) => data,
            Err(_) => {
                error!("❌ Failed to parse incoming JSON");
                return StatusCode::BAD_REQUEST;
            }
        };

        // ✅ Handle Single Log Entry
        if let Ok(log) = serde_json::from_value::<LogEntry>(json.clone()) {
            return process_log(log, &state).await;
        }

        // ✅ Handle Batch Log Entries
        if let Ok(logs) = serde_json::from_value::<Vec<LogEntry>>(json) {
            for log in logs {
                let _ = process_log(log, &state).await;
            }
            return StatusCode::OK;
        }
    }

    
    info!("✅ Received log: {:?}", payload); // ✅ Ensures log is printed


    error!("❌ Invalid JSON format: Expected single LogEntry or array");
    StatusCode::BAD_REQUEST
}

// ✅ Function to process logs safely
async fn process_log(log: LogEntry, state: &Arc<AppState>) -> StatusCode {
    info!("✅ Received log: {:?}", log);
    
    if let Err(_) = state.sender.send(log).await {
        error!("❌ Log queue is full, dropping log");
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    StatusCode::OK
}

// ✅ Batch Processing for Log Forwarding
pub async fn start_log_processor(mut receiver: mpsc::Receiver<LogEntry>, processor_url: String) {
    let mut buffer = Vec::new();

    loop {
        tokio::select! {
            Some(log) = receiver.recv() => {
                buffer.push(log);
                if buffer.len() >= 100 {
                    send_logs(std::mem::take(&mut buffer), &processor_url).await;
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)), if !buffer.is_empty() => {
                send_logs(std::mem::take(&mut buffer), &processor_url).await;
            }
        }
    }
}
