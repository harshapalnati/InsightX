use axum::{extract::State, Json, http::StatusCode};
use tokio::{sync::mpsc, time::Duration};
use tracing::{info, error};
use std::{io::Write, sync::Arc};
use tokio::time::sleep;
use crate::{models::LogEntry, forwarder::send_logs};

#[derive(Clone)]
pub struct AppState {
    pub sender: mpsc::Sender<LogEntry>,
}

// ✅ Improved: Print log and avoid panic
pub async fn ingest_log(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LogEntry>
) -> StatusCode {
    info!("✅ Received log: {:?}", payload); // ✅ Ensures log is printed

    if let Err(_) = state.sender.send(payload.clone()).await {
        error!("❌ Log queue is full, dropping log");
        return StatusCode::SERVICE_UNAVAILABLE;
    }

    // ✅ Prevent panic by handling flush errors
    std::io::stdout().flush().ok();

    StatusCode::OK
}

// ✅ Improved: Batch logs with timeout to prevent stuck logs
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
            _ = sleep(Duration::from_secs(5)), if !buffer.is_empty() => {
                // ✅ Send logs even if batch size is not reached after 5 seconds
                send_logs(std::mem::take(&mut buffer), &processor_url).await;
            }
        }
    }
}
