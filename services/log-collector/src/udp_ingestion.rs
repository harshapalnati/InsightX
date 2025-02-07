use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{info, error};
use crate::models::LogEntry;
use serde_json;
use std::net::SocketAddr;

pub async fn start_udp_listener(addr: &str, sender: mpsc::Sender<LogEntry>) {
    let socket = UdpSocket::bind(addr).await.expect("⚠️ Failed to bind UDP socket");
    let mut buf = vec![0; 1024];

    info!("📡 UDP Log Listener running on {}", addr);

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _src)) => {
                let data = String::from_utf8_lossy(&buf[..size]);
                match serde_json::from_str::<LogEntry>(&data) {
                    Ok(log) => {
                        let _ = sender.send(log).await;
                    }
                    Err(e) => {
                        error!("❌ Failed to parse UDP log: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("❌ UDP receive error: {}", e);
            }
        }
    }
}
