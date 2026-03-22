use serde::{Deserialize, Serialize};

/// Emotional state modeled using the PAD (Pleasure-Arousal-Dominance) dimensional model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Ranges from -1.0 (negative) to 1.0 (positive).
    pub valence: f64,
    /// Ranges from 0.0 (calm) to 1.0 (excited).
    pub arousal: f64,
    /// Ranges from 0.0 (submissive) to 1.0 (dominant).
    pub dominance: f64,
}

impl EmotionalState {
    /// Create a neutral emotional state.
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.3,
            dominance: 0.5,
        }
    }

    /// Update emotional state based on an event.
    pub fn update(&mut self, event: EmotionalEvent) {
        match event {
            EmotionalEvent::PositiveInteraction => {
                self.valence += 0.3;
                self.arousal += 0.1;
                self.dominance += 0.05;
            }
            EmotionalEvent::NegativeInteraction => {
                self.valence -= 0.3;
                self.arousal += 0.2;
                self.dominance -= 0.1;
            }
            EmotionalEvent::ThreatDetected => {
                self.valence -= 0.5;
                self.arousal += 0.4;
                self.dominance -= 0.2;
            }
            EmotionalEvent::GoalCompleted => {
                self.valence += 0.4;
                self.arousal += 0.2;
                self.dominance += 0.1;
            }
            EmotionalEvent::GoalFailed => {
                self.valence -= 0.3;
                self.arousal += 0.1;
                self.dominance -= 0.15;
            }
            EmotionalEvent::Idle => {
                self.arousal -= 0.1;
            }
        }
        self.clamp();
    }

    /// Decay emotional state toward neutral over time.
    /// `dt` is the time delta in seconds.
    pub fn decay(&mut self, dt: f64) {
        self.valence *= 1.0 - 0.1 * dt;
        self.arousal = self.arousal * (1.0 - 0.1 * dt) + 0.3 * 0.1 * dt;
        self.dominance = self.dominance * (1.0 - 0.1 * dt) + 0.5 * 0.1 * dt;
        self.clamp();
    }

    /// Return a human-readable mood label for the current state.
    pub fn mood_label(&self) -> &str {
        match (
            self.valence > 0.3,
            self.valence < -0.3,
            self.arousal > 0.6,
        ) {
            (true, _, false) => "content",
            (true, _, true) => "excited",
            (_, true, false) => "melancholy",
            (_, true, true) => "anxious",
            _ => "neutral",
        }
    }

    /// Generate a prompt modifier string describing the current emotional state.
    pub fn to_prompt_modifier(&self) -> String {
        format!(
            "Current mood: {}. Emotional state: valence={:.1}, arousal={:.1}.",
            self.mood_label(),
            self.valence,
            self.arousal
        )
    }

    /// Clamp all values to valid ranges.
    fn clamp(&mut self) {
        self.valence = self.valence.clamp(-1.0, 1.0);
        self.arousal = self.arousal.clamp(0.0, 1.0);
        self.dominance = self.dominance.clamp(0.0, 1.0);
    }
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self::neutral()
    }
}

/// Events that can influence the emotional state.
#[derive(Debug, Clone, PartialEq)]
pub enum EmotionalEvent {
    PositiveInteraction,
    NegativeInteraction,
    ThreatDetected,
    GoalCompleted,
    GoalFailed,
    Idle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_state_values() {
        let state = EmotionalState::neutral();
        assert_eq!(state.valence, 0.0);
        assert_eq!(state.arousal, 0.3);
        assert_eq!(state.dominance, 0.5);
    }

    #[test]
    fn neutral_mood_label() {
        let state = EmotionalState::neutral();
        assert_eq!(state.mood_label(), "neutral");
    }

    #[test]
    fn positive_interaction_increases_valence() {
        let mut state = EmotionalState::neutral();
        state.update(EmotionalEvent::PositiveInteraction);
        assert!(state.valence > 0.0);
    }

    #[test]
    fn negative_interaction_decreases_valence() {
        let mut state = EmotionalState::neutral();
        state.update(EmotionalEvent::NegativeInteraction);
        assert!(state.valence < 0.0);
    }

    #[test]
    fn threat_detected_high_arousal() {
        let mut state = EmotionalState::neutral();
        state.update(EmotionalEvent::ThreatDetected);
        assert!(state.arousal > 0.5);
        assert!(state.valence < 0.0);
    }

    #[test]
    fn goal_completed_positive() {
        let mut state = EmotionalState::neutral();
        state.update(EmotionalEvent::GoalCompleted);
        assert!(state.valence > 0.3);
        assert_eq!(state.mood_label(), "content");
    }

    #[test]
    fn goal_failed_negative() {
        let mut state = EmotionalState::neutral();
        state.update(EmotionalEvent::GoalFailed);
        assert!(state.valence < -0.2);
    }

    #[test]
    fn decay_toward_neutral() {
        let mut state = EmotionalState {
            valence: 0.8,
            arousal: 0.9,
            dominance: 0.9,
        };
        // Decay several times
        for _ in 0..100 {
            state.decay(1.0);
        }
        // Should be close to neutral
        assert!(state.valence.abs() < 0.05);
        assert!((state.arousal - 0.3).abs() < 0.05);
        assert!((state.dominance - 0.5).abs() < 0.05);
    }

    #[test]
    fn values_clamped_after_extreme_updates() {
        let mut state = EmotionalState::neutral();
        // Push valence way positive
        for _ in 0..20 {
            state.update(EmotionalEvent::GoalCompleted);
        }
        assert!(state.valence <= 1.0);
        assert!(state.arousal <= 1.0);
        assert!(state.dominance <= 1.0);

        // Push valence way negative
        for _ in 0..50 {
            state.update(EmotionalEvent::ThreatDetected);
        }
        assert!(state.valence >= -1.0);
        assert!(state.arousal >= 0.0);
        assert!(state.dominance >= 0.0);
    }

    #[test]
    fn excited_mood() {
        let state = EmotionalState {
            valence: 0.5,
            arousal: 0.8,
            dominance: 0.5,
        };
        assert_eq!(state.mood_label(), "excited");
    }

    #[test]
    fn melancholy_mood() {
        let state = EmotionalState {
            valence: -0.5,
            arousal: 0.3,
            dominance: 0.5,
        };
        assert_eq!(state.mood_label(), "melancholy");
    }

    #[test]
    fn anxious_mood() {
        let state = EmotionalState {
            valence: -0.5,
            arousal: 0.8,
            dominance: 0.5,
        };
        assert_eq!(state.mood_label(), "anxious");
    }

    #[test]
    fn prompt_modifier_contains_mood() {
        let state = EmotionalState::neutral();
        let modifier = state.to_prompt_modifier();
        assert!(modifier.contains("neutral"));
        assert!(modifier.contains("valence=0.0"));
        assert!(modifier.contains("arousal=0.3"));
    }

    #[test]
    fn idle_reduces_arousal() {
        let mut state = EmotionalState {
            valence: 0.0,
            arousal: 0.8,
            dominance: 0.5,
        };
        state.update(EmotionalEvent::Idle);
        assert!(state.arousal < 0.8);
    }

    #[test]
    fn default_is_neutral() {
        let state = EmotionalState::default();
        let neutral = EmotionalState::neutral();
        assert_eq!(state.valence, neutral.valence);
        assert_eq!(state.arousal, neutral.arousal);
        assert_eq!(state.dominance, neutral.dominance);
    }

    #[test]
    fn serialization_roundtrip() {
        let state = EmotionalState {
            valence: 0.5,
            arousal: 0.7,
            dominance: 0.3,
        };
        let json = serde_json::to_string(&state).unwrap();
        let restored: EmotionalState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.valence, state.valence);
        assert_eq!(restored.arousal, state.arousal);
        assert_eq!(restored.dominance, state.dominance);
    }
}
