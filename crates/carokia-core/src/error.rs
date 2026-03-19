use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrainError {
    #[error("sensor error: {0}")]
    Sensor(String),

    #[error("actuator error: {0}")]
    Actuator(String),

    #[error("memory error: {0}")]
    Memory(String),

    #[error("perception error: {0}")]
    Perception(String),

    #[error("language error: {0}")]
    Language(String),

    #[error("planner error: {0}")]
    Planner(String),

    #[error("decision error: {0}")]
    Decision(String),

    #[error("bus error: {0}")]
    Bus(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("internal error: {0}")]
    Internal(String),
}
