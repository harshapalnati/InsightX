use bollard::container::{ListContainersOptions, LogOutput, LogsOptions};
use bollard::Docker;
use futures_util::stream::{StreamExt, TryStreamExt};
use std::{sync::Arc};
use tokio::sync::mpsc;
use tracing::{error, info};
use crate::models::LogEntry;
use chrono::Utc;

/// Configuration for Docker log ingestion. You can extend this with additional filters.
pub struct DockerIngestionConfig {
    pub container_name: String,
}

/// Starts the Docker log ingestion process by listing containers and spawning a log monitor
/// for each container that matches the provided DockerIngestionConfig.
pub async fn start_docker_log_ingestion(
    docker_config: Arc<DockerIngestionConfig>,
    sender: mpsc::Sender<LogEntry>,
) {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            error!("❌ Failed to connect to Docker daemon: {}", e);
            return;
        }
    };

    match docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
    {
        Ok(containers) => {
            if containers.is_empty() {
                info!("⚠️ No running containers found.");
                return;
            }

            for container in containers {
                // If a container name filter is provided, skip containers that do not match.
                if let Some(names) = container.names {
                    if !names.iter().any(|name| name.contains(&docker_config.container_name)) {
                        continue;
                    }
                }
                if let Some(container_id) = container.id {
                    let docker_clone = Arc::clone(&docker);
                    let sender_clone = sender.clone();
                    tokio::spawn(async move {
                        if let Err(e) = monitor_container_logs(docker_clone, &container_id, sender_clone).await {
                            error!("❌ Error monitoring container {}: {}", container_id, e);
                        }
                    });
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to list containers: {}", e);
        }
    }
}

/// Monitors logs for a single container and sends each log entry through the provided channel.
async fn monitor_container_logs(
    docker: Arc<Docker>,
    container_id: &str,
    sender: mpsc::Sender<LogEntry>,
) -> Result<(), bollard::errors::Error> {
    info!("🐳 Watching logs for container: {}", container_id);

    let options = Some(LogsOptions::<String> {
        stdout: true,
        stderr: true,
        timestamps: true,
        follow: true,
        tail: "all".to_string(),
        ..Default::default()
    });

    // NOTE: Remove `.await` here because `docker.logs()` returns a stream.
    let mut logs_stream = docker.logs::<String>(container_id, options).fuse();

    while let Some(log) = logs_stream.next().await {
        match log {
            Ok(LogOutput::StdOut { message }) | Ok(LogOutput::StdErr { message }) => {
                if let Ok(text) = String::from_utf8(message.to_vec()) {
                    let log_entry = LogEntry {
                        source: container_id.to_string(),
                        level: "INFO".to_string(),
                        message: text.trim().to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                    };

                    if sender.send(log_entry).await.is_err() {
                        error!("❌ Log queue is full, dropping log.");
                    }
                }
            }
            Err(e) => {
                error!("❌ Error processing Docker logs: {}", e);
            }
            _ => {} // Ignore other log types
        }
    }

    Ok(())
}
