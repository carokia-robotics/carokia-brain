use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::error::BrainError;
use crate::traits::MessageBus;

/// A broadcast-based message bus keyed by topic string.
pub struct BroadcastBus {
    capacity: usize,
    channels: Mutex<HashMap<String, broadcast::Sender<Vec<u8>>>>,
}

impl BroadcastBus {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            channels: Mutex::new(HashMap::new()),
        }
    }

    fn get_or_create_sender(&self, topic: &str) -> broadcast::Sender<Vec<u8>> {
        let mut map = self.channels.lock().unwrap();
        map.entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(self.capacity).0)
            .clone()
    }
}

#[async_trait]
impl MessageBus for BroadcastBus {
    async fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<(), BrainError> {
        let sender = self.get_or_create_sender(topic);
        // It's okay if there are no receivers yet.
        let _ = sender.send(payload);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<Vec<u8>>, BrainError> {
        let sender = self.get_or_create_sender(topic);
        Ok(sender.subscribe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pub_sub_round_trip() {
        let bus = BroadcastBus::new(16);
        let mut rx = bus.subscribe("test").await.unwrap();
        bus.publish("test", b"hello".to_vec()).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, b"hello");
    }

    #[tokio::test]
    async fn publish_without_subscribers_does_not_error() {
        let bus = BroadcastBus::new(16);
        let result = bus.publish("nobody", b"data".to_vec()).await;
        assert!(result.is_ok());
    }
}
