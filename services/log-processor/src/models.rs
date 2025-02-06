use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LogEntry {
    // Optional timestamp (modify the type/format as needed)
    pub timestamp: Option<String>,
    // Optional log level (e.g., INFO, ERROR)
    pub level: Option<String>,
    // Required log message
    pub message: String,
}
