use async_trait::async_trait;
use carokia_core::{Action, ActionCommand, BrainError, Priority};
use std::sync::Mutex;

use crate::{Behavior, WorldState};

/// Internal mutable state for patrol behavior.
struct PatrolState {
    current_waypoint: usize,
}

/// Waypoint-based autonomous patrol behavior.
///
/// The robot moves toward each waypoint in sequence, cycling back to the first
/// after reaching the last. The behavior always produces a Move action toward
/// the current target waypoint.
pub struct PatrolBehavior {
    waypoints: Vec<(f64, f64)>,
    arrival_threshold: f64,
    state: Mutex<PatrolState>,
}

impl PatrolBehavior {
    pub fn new(waypoints: Vec<(f64, f64)>) -> Self {
        assert!(!waypoints.is_empty(), "PatrolBehavior requires at least one waypoint");
        Self {
            waypoints,
            arrival_threshold: 0.5,
            state: Mutex::new(PatrolState { current_waypoint: 0 }),
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.arrival_threshold = threshold;
        self
    }

    /// Returns the current target waypoint.
    pub fn current_target(&self) -> (f64, f64) {
        let state = self.state.lock().unwrap();
        self.waypoints[state.current_waypoint]
    }

    /// Returns the current waypoint index.
    pub fn current_index(&self) -> usize {
        self.state.lock().unwrap().current_waypoint
    }

    /// Returns total number of waypoints.
    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }

    /// Advance to the next waypoint (wrapping around).
    pub fn advance(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_waypoint = (state.current_waypoint + 1) % self.waypoints.len();
    }

    /// Check if the given position is within arrival threshold of the current target.
    pub fn has_arrived(&self, x: f64, y: f64) -> bool {
        let (tx, ty) = self.current_target();
        let dx = tx - x;
        let dy = ty - y;
        (dx * dx + dy * dy).sqrt() < self.arrival_threshold
    }
}

#[async_trait]
impl Behavior for PatrolBehavior {
    fn name(&self) -> &str {
        "PatrolBehavior"
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }

    async fn evaluate(&self, _state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        // Use robot position from percepts if available, otherwise use a default.
        // In the guardian demo, the robot position is passed via the WorldState's
        // first percept as a convention, but here we compute move direction generically.
        let (tx, ty) = self.current_target();

        // Always produce a Move toward the current waypoint.
        // The actual position tracking and arrival detection is done by the caller
        // (the simulation loop) which calls advance() when the robot arrives.
        Ok(Some(ActionCommand::new(
            Priority::Normal,
            Action::Move {
                x: tx,
                y: ty,
                z: 0.0,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patrol_new_and_target() {
        let patrol = PatrolBehavior::new(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)]);
        assert_eq!(patrol.current_target(), (1.0, 2.0));
        assert_eq!(patrol.current_index(), 0);
        assert_eq!(patrol.waypoint_count(), 3);
    }

    #[test]
    fn patrol_advance_cycles() {
        let patrol = PatrolBehavior::new(vec![(1.0, 2.0), (3.0, 4.0)]);
        assert_eq!(patrol.current_index(), 0);

        patrol.advance();
        assert_eq!(patrol.current_index(), 1);
        assert_eq!(patrol.current_target(), (3.0, 4.0));

        patrol.advance();
        assert_eq!(patrol.current_index(), 0);
        assert_eq!(patrol.current_target(), (1.0, 2.0));
    }

    #[test]
    fn patrol_arrival_detection() {
        let patrol = PatrolBehavior::new(vec![(5.0, 5.0)]);
        assert!(patrol.has_arrived(5.0, 5.0));
        assert!(patrol.has_arrived(5.3, 5.0)); // within 0.5
        assert!(!patrol.has_arrived(6.0, 5.0)); // too far
    }

    #[test]
    fn patrol_custom_threshold() {
        let patrol = PatrolBehavior::new(vec![(5.0, 5.0)]).with_threshold(1.0);
        assert!(patrol.has_arrived(5.8, 5.0)); // within 1.0
        assert!(!patrol.has_arrived(6.5, 5.0)); // too far
    }

    #[tokio::test]
    async fn patrol_evaluate_returns_move() {
        let patrol = PatrolBehavior::new(vec![(8.0, 3.0)]);
        let state = WorldState::new();
        let cmd = patrol.evaluate(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::Normal);
        match cmd.action {
            Action::Move { x, y, .. } => {
                assert!((x - 8.0).abs() < 1e-10);
                assert!((y - 3.0).abs() < 1e-10);
            }
            _ => panic!("Expected Move action"),
        }
    }

    #[test]
    #[should_panic(expected = "at least one waypoint")]
    fn patrol_empty_waypoints_panics() {
        PatrolBehavior::new(vec![]);
    }
}
