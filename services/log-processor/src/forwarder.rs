use reqwest::Client;
use tracing::{info, error};
use crate::models::LogEntry;
use anyhow::{Result, anyhow};

pub async fn send_logs(logs: Vec<LogEntry>, storage_url: &str) -> Result<()> {
    if logs.is_empty() {
        return Ok(());
    }

    let client = Client::new();
    info!("🚀 Sending {} logs to Storage Service: {}", logs.len(), storage_url);

    let response = client.post(storage_url)
        .header("Content-Type", "application/json")
        .json(&logs)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            info!("✅ Successfully sent logs to Storage Service");
            Ok(())
        },
        Ok(resp) => {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("❌ Storage Service responded with {}: {}", status, error_text);
            Err(anyhow!("Storage Service responded with {}: {}", status, error_text))
        },
        Err(e) => {
            error!("❌ Request to Storage Service failed: {}", e);
            Err(anyhow!(e))
        }
    }
}
