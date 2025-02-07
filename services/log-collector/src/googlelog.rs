use google_cloud_logging::{Client, ClientOptions};
use tokio::sync::mpsc;
use tracing::{error, info};
use crate::models::LogEntry;
use std::sync::Arc;
use chrono::Utc;

#[derive(Clone)]
pub struct GCPLoggingConfig {
    pub project_id: String, // GCP Project ID
}

pub async fn start_gcp_log_ingestion(
    config: Arc<GCPLoggingConfig>,
    sender: mpsc::Sender<LogEntry>,
) {
    let client = match Client::new(ClientOptions {
        project_id: config.project_id.clone(),
        ..Default::default()
    })
    .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Failed to connect to Google Cloud Logging: {}", e);
            return;
        }
    };

    loop {
        match client.list_log_entries(None, None, None).await {
            Ok(entries) => {
                for entry in entries {
                    let log_entry = LogEntry {
                        source: "GCP".to_string(),
                        level: entry.severity.unwrap_or_else(|| "INFO".to_string()),
                        message: entry.text_payload.unwrap_or_else(|| "".to_string()),
                        timestamp: entry.timestamp.unwrap_or_else(|| Utc::now().to_rfc3339()),
                    };

                    if sender.send(log_entry).await.is_err() {
                        error!("❌ Log queue full, dropping GCP log.");
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to fetch GCP logs: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await; // Poll every 10s
    }
}
