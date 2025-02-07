use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{info, error};
use crate::models::LogEntry;
use std::net::SocketAddr;
use std::str;

pub async fn start_syslog_listener(addr: &str, sender: mpsc::Sender<LogEntry>) {
    let socket = UdpSocket::bind(addr).await.expect("⚠️ Failed to bind Syslog UDP socket");
    let mut buf = vec![0; 2048];

    info!("📡 Syslog Listener running on {}", addr);

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, src)) => {
                let raw_msg = str::from_utf8(&buf[..size]).unwrap_or("<invalid UTF-8>");
                let log = parse_syslog(raw_msg, src);

                if let Some(log) = log {
                    let _ = sender.send(log).await;
                }
            }
            Err(e) => {
                error!("❌ Syslog receive error: {}", e);
            }
        }
    }
}

// ✅ Parses raw Syslog message into LogEntry struct
fn parse_syslog(msg: &str, src: SocketAddr) -> Option<LogEntry> {
    let parts: Vec<&str> = msg.splitn(3, ' ').collect();
    
    if parts.len() < 3 {
        error!("❌ Invalid Syslog format from {}: {}", src, msg);
        return None;
    }

    Some(LogEntry {
        source: format!("syslog:{}", src),
        level: parts[0].to_string(),
        message: parts[2].to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
