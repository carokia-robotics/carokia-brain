use async_trait::async_trait;

use crate::error::BrainError;
use crate::types::{ActionCommand, SensorFrame};

/// Produces sensor frames from hardware or simulation.
#[async_trait]
pub trait SensorSource: Send + Sync {
    async fn read_frame(&mut self) -> Result<SensorFrame, BrainError>;
}

/// Executes action commands on hardware or simulation.
#[async_trait]
pub trait Actuator: Send + Sync {
    async fn execute(&mut self, command: ActionCommand) -> Result<(), BrainError>;
}

/// Pub/sub message bus by string topic.
#[async_trait]
pub trait MessageBus: Send + Sync {
    async fn publish(&self, topic: &str, payload: Vec<u8>) -> Result<(), BrainError>;
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<tokio::sync::broadcast::Receiver<Vec<u8>>, BrainError>;
}
