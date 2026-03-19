use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Milliseconds since UNIX epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn now() -> Self {
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Self(ms)
    }
}

/// Sensor modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Vision,
    Audio,
    Lidar,
    Imu,
    Touch,
}

/// Raw sensor payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorPayload {
    Bytes(Vec<u8>),
    Text(String),
    Json(serde_json::Value),
}

/// A single sensor frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorFrame {
    pub id: String,
    pub timestamp: Timestamp,
    pub modality: Modality,
    pub payload: SensorPayload,
}

impl SensorFrame {
    pub fn new(modality: Modality, payload: SensorPayload) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            modality,
            payload,
        }
    }
}

/// Priority level for actions and behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Concrete action the robot can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Move { x: f64, y: f64, z: f64 },
    Actuate { joint: String, angle: f64 },
    Speak { text: String },
    Halt,
}

/// A command wrapping an action with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCommand {
    pub id: String,
    pub timestamp: Timestamp,
    pub priority: Priority,
    pub action: Action,
}

impl ActionCommand {
    pub fn new(priority: Priority, action: Action) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            priority,
            action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_now_is_positive() {
        let ts = Timestamp::now();
        assert!(ts.0 > 0);
    }

    #[test]
    fn sensor_frame_has_unique_id() {
        let a = SensorFrame::new(Modality::Vision, SensorPayload::Text("hello".into()));
        let b = SensorFrame::new(Modality::Vision, SensorPayload::Text("world".into()));
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn action_command_creation() {
        let cmd = ActionCommand::new(Priority::High, Action::Halt);
        assert_eq!(cmd.priority, Priority::High);
    }
}
