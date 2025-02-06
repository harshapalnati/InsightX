use crate::models::LogEntry;
use reqwest::Client;
use tracing::{info, error};
use lz4_flex::compress_prepend_size;

pub async fn send_logs(logs: Vec<LogEntry>, processor_url: &str) {
    if logs.is_empty() {
        return;
    }

    info!("🚀 Sending {} logs to Log Processor: {}", logs.len(), processor_url);

    let json = serde_json::to_string(&logs).unwrap();
    let compressed = compress_prepend_size(json.as_bytes()); // ✅ Compress logs before sending

    let client = Client::new();
    match client.post(processor_url)
        .header("Content-Encoding", "lz4") // ✅ Indicate LZ4 compression
        .body(compressed)
        .send()
        .await {
            Ok(_) => info!("✅ Successfully sent {} logs to Log Processor", logs.len()),
            Err(e) => error!("❌ Failed to send logs: {}", e),
        }
}
