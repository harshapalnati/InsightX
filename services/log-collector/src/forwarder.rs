use crate::models::LogEntry;
use reqwest::Client;
use tracing::{info, error};
use zstd::stream::encode_all;
use std::io::Cursor;

// ✅ Compress Logs Before Sending
pub fn compress_logs(logs: &Vec<LogEntry>) -> Vec<u8> {
    let json_logs = serde_json::to_string(logs).unwrap();
    encode_all(Cursor::new(json_logs.as_bytes()), 0).unwrap()
}

// ✅ Sends logs in batches to Log Processor
pub async fn send_logs(logs: Vec<LogEntry>, processor_url: &str) {
    if logs.is_empty() {
        return;
    }

    let compressed_logs = compress_logs(&logs);
    info!("🚀 Sending {} logs to Log Processor: {} ({} bytes)", logs.len(), processor_url, compressed_logs.len());

    let client = Client::new();
    match client.post(processor_url)
        .header("Content-Encoding", "zstd") // ✅ Indicate Compression
        .body(compressed_logs)
        .send()
        .await {
            Ok(_) => info!("✅ Successfully sent {} logs to Log Processor", logs.len()),
            Err(e) => error!("❌ Failed to send logs: {}", e),
        }
}
