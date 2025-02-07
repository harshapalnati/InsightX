use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tracing::{info, error, warn};
use std::{sync::Arc, path::PathBuf};
use crate::models::LogEntry;

#[derive(Clone)]
pub struct FileIngestionConfig {
    pub log_directory: PathBuf, // Directory to watch for log files
}

// ✅ Watches for new logs in files and processes them efficiently
pub async fn watch_log_files(
    file_config: Arc<FileIngestionConfig>,
    sender: mpsc::Sender<LogEntry>
) {
    let log_dir = file_config.log_directory.clone();
    info!("📂 Watching log directory: {:?}", log_dir);

    // ✅ Ensure log directory exists
    if !log_dir.exists() {
        warn!("⚠️ Log directory does not exist: {:?}", log_dir);
        return;
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res {
            Ok(event) => {
                if let EventKind::Modify(_) = event.kind {
                    for path in event.paths {
                        let _ = tx.try_send(path.clone()); // Non-blocking send
                    }
                }
            }
            Err(err) => error!("❌ File watcher error: {:?}", err),
        }
    }).unwrap();

    watcher.watch(&log_dir, RecursiveMode::Recursive).unwrap();

    while let Some(path) = rx.recv().await {
        if let Err(e) = ingest_log_file(path, sender.clone()).await {
            error!("❌ Failed to read log file: {}", e);
        }
    }
}

// ✅ Reads a file line-by-line and ingests logs
async fn ingest_log_file(file_path: PathBuf, sender: mpsc::Sender<LogEntry>) -> tokio::io::Result<()> {
    info!("📄 Ingesting logs from file: {:?}", file_path);

    let file = File::open(&file_path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<LogEntry>(&line) {
            Ok(log) => {
                if sender.send(log).await.is_err() {
                    error!("❌ Log queue is full, dropping log from file: {:?}", file_path);
                }
            }
            Err(_) => warn!("⚠️ Skipping malformed log entry in file: {:?}", file_path),
        }
    }

    Ok(())
}
