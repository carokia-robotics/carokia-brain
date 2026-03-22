use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrainError {
    #[error("Sensor subsystem failed: {0}")]
    Sensor(String),

    #[error("Actuator command failed: {0}")]
    Actuator(String),

    #[error("Memory store operation failed: {0}")]
    Memory(String),

    #[error("Perception pipeline failed: {0}")]
    Perception(String),

    #[error("Language backend error: {0}")]
    Language(String),

    #[error("Planner could not decompose goal: {0}")]
    Planner(String),

    #[error("Decision engine error: {0}")]
    Decision(String),

    #[error("Event bus error: {0}")]
    Bus(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
