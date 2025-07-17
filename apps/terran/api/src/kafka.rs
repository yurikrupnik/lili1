use eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone)]
pub struct KafkaClient {
    brokers: String,
    connected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerranEvent {
    pub event_type: String,
    pub resource_name: String,
    pub resource_namespace: String,
    pub timestamp: String,
    pub data: serde_json::Value,
}

impl KafkaClient {
    pub async fn new(brokers: &str) -> Result<Self> {
        info!("Creating mock Kafka client for brokers: {}", brokers);

        Ok(Self {
            brokers: brokers.to_string(),
            connected: true,
        })
    }

    pub async fn publish_event(&self, topic: &str, event: &TerranEvent) -> Result<()> {
        info!("Mock: Publishing event to topic {}: {:?}", topic, event);

        // Simulate successful event publishing
        Ok(())
    }

    pub async fn subscribe_to_topics(&self, topics: &[&str]) -> Result<()> {
        info!("Mock: Subscribing to topics: {:?}", topics);
        Ok(())
    }

    pub async fn consume_events<F>(&self, mut _handler: F) -> Result<()>
    where
        F: FnMut(TerranEvent) -> Result<()>,
    {
        info!("Mock: Starting Kafka event consumption (no-op)");

        // In a real implementation, this would consume events from Kafka
        // For now, we'll just simulate it being ready but not consuming anything

        // Keep the function running to simulate event consumption
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            info!("Mock: Kafka consumer still running...");
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        if self.connected {
            info!("Mock: Kafka health check passed");
            Ok(())
        } else {
            Err(eyre::eyre!("Mock: Kafka connection failed"))
        }
    }
}
