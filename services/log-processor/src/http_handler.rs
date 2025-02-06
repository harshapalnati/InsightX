use axum::{
    extract::State,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, error};
use crate::{models::LogEntry, forwarder::send_logs};
use lz4_flex::decompress_size_prepended;

#[derive(Clone)]
pub struct AppState {
    pub storage_service_url: String,
}

pub async fn ingest_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // If a "Content-Encoding" header is provided and equals "lz4", decompress the payload.
    let data = if let Some(encoding) = headers.get("content-encoding") {
        if encoding.to_str().unwrap_or("").eq_ignore_ascii_case("lz4") {
            match decompress_size_prepended(&body) {
                Ok(decompressed) => decompressed,
                Err(e) => {
                    error!("❌ Failed to decompress LZ4 payload: {}", e);
                    return StatusCode::BAD_REQUEST;
                }
            }
        } else {
            body.to_vec()
        }
    } else {
        body.to_vec()
    };

    // Parse the (possibly decompressed) data as JSON.
    let json_payload: Value = match serde_json::from_slice(&data) {
        Ok(val) => val,
        Err(e) => {
            error!("❌ Failed to parse JSON: {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    info!("📥 Incoming request: {:?}", json_payload);

    // Attempt to parse the JSON payload as an array of logs; if that fails, try parsing a single log entry.
    let logs: Vec<LogEntry> = if let Ok(logs) = serde_json::from_value(json_payload.clone()) {
        logs
    } else if let Ok(single_log) = serde_json::from_value::<LogEntry>(json_payload) {
        vec![single_log]
    } else {
        error!("❌ Invalid log format");
        return StatusCode::BAD_REQUEST;
    };

    info!("✅ Received {} logs for processing", logs.len());

    // Forward the logs to the storage service.
    if let Err(e) = send_logs(logs, &state.storage_service_url).await {
        error!("❌ Failed to forward logs: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
