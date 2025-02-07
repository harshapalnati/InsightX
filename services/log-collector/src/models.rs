use chrono::{Utc, DateTime};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub source: String,
    pub level: String,
    pub message: String,
    #[serde(default = "default_timestamp")]  // ✅ Auto-fill timestamp if missing
    pub timestamp: String,
}

fn default_timestamp() -> String {
    Utc::now().to_rfc3339()
}
