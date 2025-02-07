use azure_sdk_loganalytics::{Client, QueryResponse};
use tokio::sync::mpsc;
use tracing::{error, info};
use crate::models::LogEntry;
use std::sync::Arc;
use chrono::Utc;

#[derive(Clone)]
pub struct AzureLogConfig {
    pub workspace_id: String,
    pub query: String, // Log Query
}

pub async fn start_azure_log_ingestion(
    config: Arc<AzureLogConfig>,
    sender: mpsc::Sender<LogEntry>,
) {
    let client = match Client::new(&config.workspace_id, None) {
        Ok(c) => c,
        Err(e) => {
            error!("❌ Failed to connect to Azure Logs: {}", e);
            return;
        }
    };

    loop {
        match client.query(&config.query).await {
            Ok(QueryResponse::Json(json)) => {
                if let Ok(entries) = serde_json::from_value::<Vec<LogEntry>>(json) {
                    for entry in entries {
                        if sender.send(entry).await.is_err() {
                            error!("❌ Log queue full, dropping Azure log.");
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to fetch Azure logs: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await; // Poll every 10s
    }
}
