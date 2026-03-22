pub mod emergency;
pub mod patrol;
pub mod threat;

use async_trait::async_trait;
use carokia_core::{Action, ActionCommand, BrainError, Priority};
use carokia_memory::MemoryEntry;
use carokia_perception::Percept;
use carokia_planner::{Goal, TaskNode, TaskStatus};

pub use emergency::EmergencyResponseBehavior;
pub use patrol::PatrolBehavior;
pub use threat::ThreatDetectionBehavior;

/// Threat level for the guardian system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThreatLevel {
    None = 0,
    Suspicious = 1,
    Confirmed = 2,
}

impl std::fmt::Display for ThreatLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThreatLevel::None => write!(f, "NONE"),
            ThreatLevel::Suspicious => write!(f, "SUSPICIOUS"),
            ThreatLevel::Confirmed => write!(f, "CONFIRMED"),
        }
    }
}

/// Aggregated world state for decision-making.
#[derive(Debug, Clone)]
pub struct WorldState {
    pub percepts: Vec<Percept>,
    pub goals: Vec<Goal>,
    pub tasks: Vec<TaskNode>,
    pub memories: Vec<MemoryEntry>,
    pub tick: u64,
    pub threat_level: ThreatLevel,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            percepts: Vec::new(),
            goals: Vec::new(),
            tasks: Vec::new(),
            memories: Vec::new(),
            tick: 0,
            threat_level: ThreatLevel::None,
        }
    }

    /// Returns true if any percept indicates an obstacle within the given distance.
    pub fn has_close_obstacle(&self, threshold: f64) -> bool {
        use carokia_perception::PerceptContent;
        self.percepts.iter().any(|p| {
            matches!(&p.content, PerceptContent::Obstacle { distance, .. } if *distance < threshold)
        })
    }

    /// Returns pending tasks sorted by goal priority (descending).
    pub fn pending_tasks(&self) -> Vec<&TaskNode> {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .collect()
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// A named behavior that can evaluate the world and propose an action.
#[async_trait]
pub trait Behavior: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> Priority;
    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError>;
}

// ---------- Built-in behaviors ----------

pub struct EmergencyHalt;

#[async_trait]
impl Behavior for EmergencyHalt {
    fn name(&self) -> &str {
        "EmergencyHalt"
    }
    fn priority(&self) -> Priority {
        Priority::Critical
    }
    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        if state.has_close_obstacle(0.3) {
            Ok(Some(ActionCommand::new(Priority::Critical, Action::Halt)))
        } else {
            Ok(None)
        }
    }
}

pub struct ReactiveAvoidance;

#[async_trait]
impl Behavior for ReactiveAvoidance {
    fn name(&self) -> &str {
        "ReactiveAvoidance"
    }
    fn priority(&self) -> Priority {
        Priority::High
    }
    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        if state.has_close_obstacle(1.0) && !state.has_close_obstacle(0.3) {
            // Steer away.
            Ok(Some(ActionCommand::new(
                Priority::High,
                Action::Move {
                    x: -0.5,
                    y: 0.5,
                    z: 0.0,
                },
            )))
        } else {
            Ok(None)
        }
    }
}

pub struct ExecuteTask;

#[async_trait]
impl Behavior for ExecuteTask {
    fn name(&self) -> &str {
        "ExecuteTask"
    }
    fn priority(&self) -> Priority {
        Priority::Normal
    }
    async fn evaluate(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        if let Some(task) = state.pending_tasks().first() {
            if let Some(ref action) = task.action {
                return Ok(Some(ActionCommand::new(Priority::Normal, action.clone())));
            }
        }
        Ok(None)
    }
}

pub struct Idle;

#[async_trait]
impl Behavior for Idle {
    fn name(&self) -> &str {
        "Idle"
    }
    fn priority(&self) -> Priority {
        Priority::Low
    }
    async fn evaluate(&self, _state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        // Always available as fallback.
        Ok(Some(ActionCommand::new(Priority::Low, Action::Halt)))
    }
}

/// Decision engine that selects the highest-priority behavior that fires.
pub struct BehaviorDecisionEngine {
    behaviors: Vec<Box<dyn Behavior>>,
}

impl BehaviorDecisionEngine {
    pub fn new() -> Self {
        Self {
            behaviors: Vec::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut engine = Self::new();
        engine.add_behavior(Box::new(EmergencyHalt));
        engine.add_behavior(Box::new(ReactiveAvoidance));
        engine.add_behavior(Box::new(ExecuteTask));
        engine.add_behavior(Box::new(Idle));
        engine
    }

    pub fn add_behavior(&mut self, behavior: Box<dyn Behavior>) {
        self.behaviors.push(behavior);
        // Sort by priority descending so highest-priority behaviors are evaluated first.
        self.behaviors.sort_by_key(|b| std::cmp::Reverse(b.priority()));
    }

    /// Evaluate all behaviors and return the command from the highest-priority one that fires.
    pub async fn tick(&self, state: &WorldState) -> Result<Option<ActionCommand>, BrainError> {
        for behavior in &self.behaviors {
            if let Some(cmd) = behavior.evaluate(state).await? {
                tracing::debug!(behavior = behavior.name(), "Behavior fired");
                return Ok(Some(cmd));
            }
        }
        Ok(None)
    }
}

impl Default for BehaviorDecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_core::Modality;
    use carokia_perception::{Percept, PerceptContent};

    fn obstacle_percept(distance: f64) -> Percept {
        Percept::new(
            Modality::Lidar,
            PerceptContent::Obstacle {
                distance,
                bearing: 0.0,
            },
            0.9,
        )
    }

    #[tokio::test]
    async fn emergency_halt_triggers_on_close_obstacle() {
        let engine = BehaviorDecisionEngine::with_defaults();
        let mut state = WorldState::new();
        state.percepts.push(obstacle_percept(0.2));
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::Critical);
        assert!(matches!(cmd.action, Action::Halt));
    }

    #[tokio::test]
    async fn reactive_avoidance_at_medium_distance() {
        let engine = BehaviorDecisionEngine::with_defaults();
        let mut state = WorldState::new();
        state.percepts.push(obstacle_percept(0.7));
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::High);
    }

    #[tokio::test]
    async fn idle_when_no_stimuli() {
        let engine = BehaviorDecisionEngine::with_defaults();
        let state = WorldState::new();
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::Low);
    }

    #[tokio::test]
    async fn execute_task_picks_pending() {
        let engine = BehaviorDecisionEngine::with_defaults();
        let mut state = WorldState::new();
        let mut task = carokia_planner::TaskNode::new("g1", "Do thing");
        task.action = Some(Action::Speak {
            text: "hello".into(),
        });
        state.tasks.push(task);
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::Normal);
        assert!(matches!(cmd.action, Action::Speak { .. }));
    }

    // --- ThreatLevel tests ---

    #[test]
    fn threat_level_ordering() {
        assert!(ThreatLevel::None < ThreatLevel::Suspicious);
        assert!(ThreatLevel::Suspicious < ThreatLevel::Confirmed);
        assert!(ThreatLevel::None < ThreatLevel::Confirmed);
    }

    #[test]
    fn threat_level_display() {
        assert_eq!(format!("{}", ThreatLevel::None), "NONE");
        assert_eq!(format!("{}", ThreatLevel::Suspicious), "SUSPICIOUS");
        assert_eq!(format!("{}", ThreatLevel::Confirmed), "CONFIRMED");
    }

    #[test]
    fn threat_level_equality() {
        assert_eq!(ThreatLevel::None, ThreatLevel::None);
        assert_ne!(ThreatLevel::None, ThreatLevel::Confirmed);
    }

    // --- Integration: full behavior priority chain ---

    #[tokio::test]
    async fn integration_patrol_to_detect_to_emergency() {
        // Set up engine with guardian behaviors
        let mut engine = BehaviorDecisionEngine::new();
        engine.add_behavior(Box::new(EmergencyHalt));
        engine.add_behavior(Box::new(EmergencyResponseBehavior::new()));
        engine.add_behavior(Box::new(ThreatDetectionBehavior::new(5.0, 2)));
        engine.add_behavior(Box::new(PatrolBehavior::new(vec![(5.0, 5.0)])));
        engine.add_behavior(Box::new(Idle));

        // Phase 1: No threat — patrol fires (Normal priority, below High behaviors that don't fire)
        let state = WorldState::new();
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::Normal); // PatrolBehavior

        // Phase 2: Unknown person detected — ThreatDetection fires (High)
        let mut state = WorldState::new();
        state.percepts.push(Percept::new(
            Modality::Vision,
            PerceptContent::Person { name: None, distance: 3.0, bearing: 0.0 },
            0.9,
        ));
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::High); // ThreatDetection (Suspicious)

        // Phase 3: Sustained detection — still ThreatDetection but now Confirmed
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::High);

        // Phase 4: With confirmed threat level in WorldState — EmergencyResponse fires
        let mut state = WorldState::new();
        state.threat_level = ThreatLevel::Confirmed;
        let cmd = engine.tick(&state).await.unwrap().unwrap();
        assert_eq!(cmd.priority, Priority::High);
        assert!(matches!(cmd.action, Action::Halt)); // EmergencyResponse halts
    }
}
