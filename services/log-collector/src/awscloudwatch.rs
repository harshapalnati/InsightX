use rusoto_core::Region;
use rusoto_logs::{CloudWatchLogs, CloudWatchLogsClient, FilterLogEventsRequest};
use tokio::sync::mpsc;
use tracing::{error, info};
use crate::models::LogEntry;
use std::sync::Arc;
use chrono::Utc;

#[derive(Clone)]
pub struct AWSCloudWatchConfig {
    pub log_group_name: String, // CloudWatch Log Group
    pub log_stream_name: Option<String>, // Optional: Specific Log Stream
}

pub async fn start_aws_log_ingestion(
    config: Arc<AWSCloudWatchConfig>,
    sender: mpsc::Sender<LogEntry>,
) {
    let client = CloudWatchLogsClient::new(Region::default());

    loop {
        let request = FilterLogEventsRequest {
            log_group_name: config.log_group_name.clone(),
            log_stream_names: config.log_stream_name.clone().map(|s| vec![s]),
            limit: Some(50), // Fetch up to 50 logs at a time
            ..Default::default()
        };

        match client.filter_log_events(request).await {
            Ok(response) => {
                if let Some(events) = response.events {
                    for event in events {
                        if let Some(message) = event.message {
                            let log_entry = LogEntry {
                                source: config.log_group_name.clone(),
                                level: "INFO".to_string(),
                                message,
                                timestamp: event.timestamp.map_or_else(
                                    || Utc::now().to_rfc3339(),
                                    |ts| chrono::TimeZone::timestamp_millis(&Utc, ts).to_rfc3339(),
                                ),
                            };

                            if sender.send(log_entry).await.is_err() {
                                error!("❌ Log queue full, dropping AWS log.");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to fetch AWS logs: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await; // Poll every 10s
    }
}
