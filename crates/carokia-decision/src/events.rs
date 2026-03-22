use async_trait::async_trait;
use carokia_core::{Action, ActionCommand, BrainError, Priority};
use carokia_perception::PerceptContent;

use crate::{Behavior, WorldState};

/// Events that can trigger reactive behaviors.
#[derive(Debug, Clone, PartialEq)]
pub enum EventTrigger {
    PersonDetected,
    UnknownFaceDetected,
    LoudNoise,
    TimerElapsed { name: String },
    Custom(String),
}

/// Associates a trigger with an action and priority.
pub struct EventReaction {
    pub trigger: EventTrigger,
    pub action: Action,
    pub priority: Priority,
}

/// Event-driven behavior that checks percepts against registered triggers.
pub struct EventDrivenBehavior {
    reactions: Vec<EventReaction>,
}

impl EventDrivenBehavior {
    pub fn new() -> Self {
        Self {
            reactions: Vec::new(),
        }
    }

    /// Register a new event reaction.
    pub fn add_reaction(&mut self, reaction: EventReaction) {
        self.reactions.push(reaction);
    }

    /// Check if a trigger matches the current world state percepts.
    fn trigger_matches(trigger: &EventTrigger, state: &WorldState) -> bool {
        match trigger {
            EventTrigger::PersonDetected => state.percepts.iter().any(|p| {
                matches!(&p.content, PerceptContent::Person { .. })
            }),
            EventTrigger::UnknownFaceDetected => state.percepts.iter().any(|p| {
                matches!(&p.content, PerceptContent::Person { name: None, .. })
            }),
            EventTrigger::LoudNoise => state.percepts.iter().any(|p| {
                matches!(&p.content, PerceptContent::AudioEvent { kind } if kind == "loud_noise")
            }),
            EventTrigger::TimerElapsed { name } => {
                // Timer events would be modeled as custom percepts
                state.percepts.iter().any(|p| {
                    matches!(&p.content, PerceptContent::AudioEvent { kind } if kind == &format!("timer:{name}"))
                })
            }
            EventTrigger::Custom(event_name) => state.percepts.iter().any(|p| {
                matches!(&p.content, PerceptContent::AudioEvent { kind } if kind == event_name)
            }),
        }
    }
}

impl Default for EventDrivenBehavior {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Behavior for EventDrivenBehavior {
    fn name(&self) -> &str {
        "EventDriven"
    }

    fn priority(&self) -> Priority {
        // Return the highest priority among registered reactions,
        // or Low if none are registered.
        self.reactions
            .iter()
            .map(|r| r.priority)
            .max()
            .unwrap_or(Priority::Low)
    }

    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        // Find the highest-priority reaction whose trigger matches.
        let mut best: Option<&EventReaction> = None;
        for reaction in &self.reactions {
            if Self::trigger_matches(&reaction.trigger, state) {
                match best {
                    Some(current) if reaction.priority > current.priority => {
                        best = Some(reaction);
                    }
                    None => {
                        best = Some(reaction);
                    }
                    _ => {}
                }
            }
        }

        if let Some(reaction) = best {
            Ok(Some(ActionCommand::new(
                reaction.priority,
                reaction.action.clone(),
            )))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_core::Modality;
    use carokia_perception::Percept;

    fn person_percept(name: Option<&str>) -> Percept {
        Percept::new(
            Modality::Vision,
            PerceptContent::Person {
                name: name.map(|s| s.to_string()),
                distance: 2.0,
                bearing: 0.0,
            },
            0.9,
        )
    }

    fn audio_percept(kind: &str) -> Percept {
        Percept::new(
            Modality::Audio,
            PerceptContent::AudioEvent {
                kind: kind.to_string(),
            },
            0.8,
        )
    }

    #[test]
    fn event_trigger_equality() {
        assert_eq!(EventTrigger::PersonDetected, EventTrigger::PersonDetected);
        assert_ne!(EventTrigger::PersonDetected, EventTrigger::LoudNoise);
        assert_eq!(
            EventTrigger::TimerElapsed { name: "a".into() },
            EventTrigger::TimerElapsed { name: "a".into() }
        );
        assert_ne!(
            EventTrigger::TimerElapsed { name: "a".into() },
            EventTrigger::TimerElapsed { name: "b".into() }
        );
        assert_eq!(
            EventTrigger::Custom("x".into()),
            EventTrigger::Custom("x".into())
        );
    }

    #[test]
    fn trigger_matches_person_detected() {
        let mut state = WorldState::new();
        state.percepts.push(person_percept(Some("Alice")));
        assert!(EventDrivenBehavior::trigger_matches(
            &EventTrigger::PersonDetected,
            &state
        ));
    }

    #[test]
    fn trigger_matches_unknown_face() {
        let mut state = WorldState::new();
        state.percepts.push(person_percept(None));
        assert!(EventDrivenBehavior::trigger_matches(
            &EventTrigger::UnknownFaceDetected,
            &state
        ));
    }

    #[test]
    fn trigger_does_not_match_known_face_for_unknown() {
        let mut state = WorldState::new();
        state.percepts.push(person_percept(Some("Bob")));
        assert!(!EventDrivenBehavior::trigger_matches(
            &EventTrigger::UnknownFaceDetected,
            &state
        ));
    }

    #[test]
    fn trigger_matches_loud_noise() {
        let mut state = WorldState::new();
        state.percepts.push(audio_percept("loud_noise"));
        assert!(EventDrivenBehavior::trigger_matches(
            &EventTrigger::LoudNoise,
            &state
        ));
    }

    #[test]
    fn trigger_no_match_on_empty_state() {
        let state = WorldState::new();
        assert!(!EventDrivenBehavior::trigger_matches(
            &EventTrigger::PersonDetected,
            &state
        ));
        assert!(!EventDrivenBehavior::trigger_matches(
            &EventTrigger::LoudNoise,
            &state
        ));
    }

    #[test]
    fn trigger_matches_custom_event() {
        let mut state = WorldState::new();
        state.percepts.push(audio_percept("doorbell"));
        assert!(EventDrivenBehavior::trigger_matches(
            &EventTrigger::Custom("doorbell".into()),
            &state
        ));
    }

    #[tokio::test]
    async fn event_driven_behavior_fires_on_matching_percept() {
        let mut behavior = EventDrivenBehavior::new();
        behavior.add_reaction(EventReaction {
            trigger: EventTrigger::PersonDetected,
            action: Action::Speak {
                text: "Hello!".into(),
            },
            priority: Priority::Normal,
        });

        let mut state = WorldState::new();
        state.percepts.push(person_percept(Some("Alice")));

        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_some());
        let cmd = result.unwrap();
        assert_eq!(cmd.priority, Priority::Normal);
        assert!(matches!(cmd.action, Action::Speak { .. }));
    }

    #[tokio::test]
    async fn event_driven_behavior_none_when_no_match() {
        let mut behavior = EventDrivenBehavior::new();
        behavior.add_reaction(EventReaction {
            trigger: EventTrigger::LoudNoise,
            action: Action::Halt,
            priority: Priority::High,
        });

        let state = WorldState::new();
        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn event_driven_behavior_picks_highest_priority() {
        let mut behavior = EventDrivenBehavior::new();
        behavior.add_reaction(EventReaction {
            trigger: EventTrigger::PersonDetected,
            action: Action::Speak {
                text: "Low priority".into(),
            },
            priority: Priority::Low,
        });
        behavior.add_reaction(EventReaction {
            trigger: EventTrigger::PersonDetected,
            action: Action::Speak {
                text: "High priority".into(),
            },
            priority: Priority::High,
        });

        let mut state = WorldState::new();
        state.percepts.push(person_percept(Some("Test")));

        let result = behavior.evaluate(&state).await.unwrap();
        assert!(result.is_some());
        let cmd = result.unwrap();
        assert_eq!(cmd.priority, Priority::High);
    }
}
