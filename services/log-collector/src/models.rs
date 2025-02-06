use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogEntry {
    pub source: String,
    pub level: String,
    pub message: String,
}
