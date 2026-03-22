use async_trait::async_trait;
use carokia_core::{Action, ActionCommand, BrainError, Priority};

use crate::{Behavior, ThreatLevel, WorldState};

/// Emergency response behavior that activates when a confirmed threat is present.
///
/// When the world state contains a confirmed threat level, this behavior
/// overrides normal operations (like patrol) by emitting a Halt action
/// at High priority with an alert message.
pub struct EmergencyResponseBehavior {
    /// The threat level that must be present in the WorldState to activate.
    trigger_level: ThreatLevel,
}

impl EmergencyResponseBehavior {
    pub fn new() -> Self {
        Self {
            trigger_level: ThreatLevel::Confirmed,
        }
    }

    /// Create an emergency response that triggers at a specific threat level.
    pub fn with_trigger(trigger_level: ThreatLevel) -> Self {
        Self { trigger_level }
    }
}

impl Default for EmergencyResponseBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Behavior for EmergencyResponseBehavior {
    fn name(&self) -> &str {
        "EmergencyResponse"
    }

    fn priority(&self) -> Priority {
        Priority::High
    }

    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        if state.threat_level >= self.trigger_level {
            Ok(Some(ActionCommand::new(Priority::High, Action::Halt)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_threat_no_response() {
        let behavior = EmergencyResponseBehavior::new();
        let state = WorldState::new();
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn suspicious_no_response_by_default() {
        let behavior = EmergencyResponseBehavior::new();
        let mut state = WorldState::new();
        state.threat_level = ThreatLevel::Suspicious;
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn confirmed_threat_triggers_halt() {
        let behavior = EmergencyResponseBehavior::new();
        let mut state = WorldState::new();
        state.threat_level = ThreatLevel::Confirmed;
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_some());
        let cmd = result.unwrap();
        assert_eq!(cmd.priority, Priority::High);
        assert!(matches!(cmd.action, Action::Halt));
    }

    #[tokio::test]
    async fn custom_trigger_at_suspicious() {
        let behavior = EmergencyResponseBehavior::with_trigger(ThreatLevel::Suspicious);
        let mut state = WorldState::new();
        state.threat_level = ThreatLevel::Suspicious;
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn custom_trigger_none_does_not_activate() {
        let behavior = EmergencyResponseBehavior::with_trigger(ThreatLevel::Suspicious);
        let state = WorldState::new();
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_none());
    }
}
