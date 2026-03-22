use async_trait::async_trait;
use carokia_core::{Action, BrainError, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "llm")]
pub mod llm_planner;

#[cfg(feature = "llm")]
pub use llm_planner::LlmPlanner;

#[cfg(feature = "reasoning")]
pub mod reasoning;

/// A high-level goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub created_at: Timestamp,
}

impl Goal {
    pub fn new(description: impl Into<String>, priority: u8) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            priority,
            created_at: Timestamp::now(),
        }
    }
}

/// Status of a task node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A single task node in a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub goal_id: String,
    pub description: String,
    pub status: TaskStatus,
    pub action: Option<Action>,
    pub depends_on: Vec<String>,
}

impl TaskNode {
    pub fn new(goal_id: &str, description: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            goal_id: goal_id.to_string(),
            description: description.into(),
            status: TaskStatus::Pending,
            action: None,
            depends_on: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }
}

/// Trait for planning: decomposing goals into task trees.
#[async_trait]
pub trait Planner: Send + Sync {
    async fn decompose(&self, goal: &Goal) -> Result<Vec<TaskNode>, BrainError>;
    async fn replan(
        &self,
        goal: &Goal,
        failed_tasks: &[TaskNode],
    ) -> Result<Vec<TaskNode>, BrainError>;
}

/// A simple rule-based planner with hardcoded decompositions.
pub struct RulePlanner;

impl RulePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RulePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Planner for RulePlanner {
    async fn decompose(&self, goal: &Goal) -> Result<Vec<TaskNode>, BrainError> {
        let desc = goal.description.to_lowercase();

        if desc.contains("move") || desc.contains("go to") || desc.contains("navigate") {
            Ok(vec![
                TaskNode::new(&goal.id, "Plan path").with_action(Action::Halt),
                TaskNode::new(&goal.id, "Execute movement").with_action(Action::Move {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }),
            ])
        } else if desc.contains("say") || desc.contains("speak") || desc.contains("tell") {
            Ok(vec![TaskNode::new(&goal.id, "Generate speech")
                .with_action(Action::Speak {
                    text: goal.description.clone(),
                })])
        } else if desc.contains("stop") || desc.contains("halt") {
            Ok(vec![
                TaskNode::new(&goal.id, "Halt all motion").with_action(Action::Halt)
            ])
        } else {
            // Default: single task echoing the goal.
            Ok(vec![TaskNode::new(&goal.id, &goal.description)])
        }
    }

    async fn replan(
        &self,
        goal: &Goal,
        _failed_tasks: &[TaskNode],
    ) -> Result<Vec<TaskNode>, BrainError> {
        // Simple strategy: just re-decompose.
        self.decompose(goal).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn decompose_movement_goal() {
        let planner = RulePlanner::new();
        let goal = Goal::new("Move to the kitchen", 5);
        let tasks = planner.decompose(&goal).await.unwrap();
        assert!(tasks.len() >= 2);
        assert!(tasks.iter().any(|t| t.action.is_some()));
    }

    #[tokio::test]
    async fn decompose_speech_goal() {
        let planner = RulePlanner::new();
        let goal = Goal::new("Say hello to the user", 3);
        let tasks = planner.decompose(&goal).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn decompose_unknown_goal() {
        let planner = RulePlanner::new();
        let goal = Goal::new("Do something mysterious", 1);
        let tasks = planner.decompose(&goal).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn replan_produces_tasks() {
        let planner = RulePlanner::new();
        let goal = Goal::new("Navigate to room B", 5);
        let tasks = planner.replan(&goal, &[]).await.unwrap();
        assert!(!tasks.is_empty());
    }
}
