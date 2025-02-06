use crate::models::LogEntry;
use tracing::info;
use rayon::prelude::*;

pub fn process_logs(mut logs: Vec<LogEntry>) -> Vec<LogEntry> {
    logs.par_iter_mut().for_each(|log| {
        log.message = log.message.trim().to_string();
        log.level = log.level.to_uppercase();
    });

    info!("🔄 Processed {} logs in parallel", logs.len());
    logs
}
