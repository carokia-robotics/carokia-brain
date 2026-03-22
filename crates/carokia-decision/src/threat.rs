use async_trait::async_trait;
use carokia_core::{Action, ActionCommand, BrainError, Priority};
use carokia_perception::PerceptContent;
use std::sync::Mutex;

use crate::{Behavior, ThreatLevel, WorldState};

/// Internal mutable state for threat detection.
struct ThreatState {
    tick_counter: usize,
    current_level: ThreatLevel,
}

/// Detects unknown persons and escalates through threat levels.
///
/// When an unknown person is detected within `alert_distance`, the internal
/// counter increments. After `sustained_ticks` consecutive detections the
/// threat is confirmed. If no person is detected the counter resets.
pub struct ThreatDetectionBehavior {
    alert_distance: f64,
    sustained_ticks: usize,
    state: Mutex<ThreatState>,
}

impl ThreatDetectionBehavior {
    pub fn new(alert_distance: f64, sustained_ticks: usize) -> Self {
        Self {
            alert_distance,
            sustained_ticks,
            state: Mutex::new(ThreatState {
                tick_counter: 0,
                current_level: ThreatLevel::None,
            }),
        }
    }

    /// Returns the current threat level.
    pub fn threat_level(&self) -> ThreatLevel {
        self.state.lock().unwrap().current_level
    }

    /// Returns the current tick counter.
    pub fn tick_counter(&self) -> usize {
        self.state.lock().unwrap().tick_counter
    }

    /// Reset the threat detector to its initial state.
    pub fn reset(&self) {
        let mut s = self.state.lock().unwrap();
        s.tick_counter = 0;
        s.current_level = ThreatLevel::None;
    }
}

#[async_trait]
impl Behavior for ThreatDetectionBehavior {
    fn name(&self) -> &str {
        "ThreatDetection"
    }

    fn priority(&self) -> Priority {
        Priority::High
    }

    async fn evaluate(&self, world: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        // Check percepts for unknown persons within alert distance.
        let person_detected = world.percepts.iter().any(|p| {
            matches!(
                &p.content,
                PerceptContent::Person { name, distance, .. }
                    if name.is_none() && *distance < self.alert_distance
            )
        });

        let mut state = self.state.lock().unwrap();

        if person_detected {
            state.tick_counter += 1;

            if state.tick_counter >= self.sustained_ticks {
                state.current_level = ThreatLevel::Confirmed;
                return Ok(Some(ActionCommand::new(
                    Priority::High,
                    Action::Speak {
                        text: "ALERT: Unknown person detected — threat confirmed!".into(),
                    },
                )));
            } else {
                state.current_level = ThreatLevel::Suspicious;
                return Ok(Some(ActionCommand::new(
                    Priority::High,
                    Action::Speak {
                        text: "Caution: Unknown person detected nearby.".into(),
                    },
                )));
            }
        } else {
            // No person detected — decay counter (but don't drop immediately).
            if state.tick_counter > 0 {
                state.tick_counter = state.tick_counter.saturating_sub(1);
            }
            if state.tick_counter == 0 {
                state.current_level = ThreatLevel::None;
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_core::Modality;
    use carokia_perception::{Percept, PerceptContent};

    fn unknown_person_percept(distance: f64) -> Percept {
        Percept::new(
            Modality::Vision,
            PerceptContent::Person {
                name: None,
                distance,
                bearing: 0.0,
            },
            0.9,
        )
    }

    fn known_person_percept(distance: f64) -> Percept {
        Percept::new(
            Modality::Vision,
            PerceptContent::Person {
                name: Some("Alice".into()),
                distance,
                bearing: 0.0,
            },
            0.9,
        )
    }

    #[tokio::test]
    async fn no_person_no_threat() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let state = WorldState::new();
        let result = detector.evaluate(&state).await.unwrap();
        assert!(result.is_none());
        assert_eq!(detector.threat_level(), ThreatLevel::None);
    }

    #[tokio::test]
    async fn known_person_no_threat() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let mut state = WorldState::new();
        state.percepts.push(known_person_percept(3.0));
        let result = detector.evaluate(&state).await.unwrap();
        assert!(result.is_none());
        assert_eq!(detector.threat_level(), ThreatLevel::None);
    }

    #[tokio::test]
    async fn unknown_person_far_away_no_threat() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let mut state = WorldState::new();
        state.percepts.push(unknown_person_percept(10.0));
        let result = detector.evaluate(&state).await.unwrap();
        assert!(result.is_none());
        assert_eq!(detector.threat_level(), ThreatLevel::None);
    }

    #[tokio::test]
    async fn unknown_person_close_becomes_suspicious() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let mut state = WorldState::new();
        state.percepts.push(unknown_person_percept(3.0));

        let result = detector.evaluate(&state).await.unwrap();
        assert!(result.is_some());
        assert_eq!(detector.threat_level(), ThreatLevel::Suspicious);
        assert_eq!(detector.tick_counter(), 1);
    }

    #[tokio::test]
    async fn sustained_detection_confirms_threat() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let mut state = WorldState::new();
        state.percepts.push(unknown_person_percept(3.0));

        // Tick 1 & 2: suspicious
        detector.evaluate(&state).await.unwrap();
        detector.evaluate(&state).await.unwrap();
        assert_eq!(detector.threat_level(), ThreatLevel::Suspicious);

        // Tick 3: confirmed
        let result = detector.evaluate(&state).await.unwrap();
        assert!(result.is_some());
        assert_eq!(detector.threat_level(), ThreatLevel::Confirmed);

        if let Some(cmd) = result {
            match &cmd.action {
                Action::Speak { text } => assert!(text.contains("confirmed")),
                _ => panic!("Expected Speak action"),
            }
        }
    }

    #[tokio::test]
    async fn counter_decays_without_person() {
        let detector = ThreatDetectionBehavior::new(5.0, 5);
        let mut state_with = WorldState::new();
        state_with.percepts.push(unknown_person_percept(3.0));

        // Build up counter
        detector.evaluate(&state_with).await.unwrap();
        detector.evaluate(&state_with).await.unwrap();
        assert_eq!(detector.tick_counter(), 2);

        // No person — counter decays
        let state_empty = WorldState::new();
        detector.evaluate(&state_empty).await.unwrap();
        assert_eq!(detector.tick_counter(), 1);

        detector.evaluate(&state_empty).await.unwrap();
        assert_eq!(detector.tick_counter(), 0);
        assert_eq!(detector.threat_level(), ThreatLevel::None);
    }

    #[tokio::test]
    async fn reset_clears_state() {
        let detector = ThreatDetectionBehavior::new(5.0, 3);
        let mut state = WorldState::new();
        state.percepts.push(unknown_person_percept(3.0));

        detector.evaluate(&state).await.unwrap();
        assert_eq!(detector.tick_counter(), 1);

        detector.reset();
        assert_eq!(detector.tick_counter(), 0);
        assert_eq!(detector.threat_level(), ThreatLevel::None);
    }
}
