use tokio::sync::mpsc;
use tracing::{info, error};
use crate::models::LogEntry;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::config::ClientConfig;
use rdkafka::message::Message;
use serde_json;

pub async fn start_kafka_listener(brokers: &str, topic: &str, group_id: &str, sender: mpsc::Sender<LogEntry>) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("⚠️ Failed to create Kafka consumer");

    consumer.subscribe(&[topic]).expect("⚠️ Failed to subscribe to topic");

    info!("📡 Kafka Listener subscribed to topic: {}", topic);

    while let Ok(message) = consumer.recv().await {
        if let Some(payload) = message.payload() {
            let log_str = String::from_utf8_lossy(payload);
            match serde_json::from_str::<LogEntry>(&log_str) {
                Ok(log) => {
                    let _ = sender.send(log).await;
                }
                Err(e) => {
                    error!("❌ Failed to parse Kafka log: {}", e);
                }
            }
        }
    }
}
