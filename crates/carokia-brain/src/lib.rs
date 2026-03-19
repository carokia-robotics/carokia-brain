use std::time::Duration;

use carokia_core::{Action, BrainError, Modality, SensorFrame, SensorPayload};
use carokia_decision::{BehaviorDecisionEngine, WorldState};
use carokia_language::{ConversationManager, MockBackend};
use carokia_memory::{MemoryEntry, MemoryKind, MemoryStore, ShortTermMemory};
use carokia_perception::{PerceptionPipeline, StubProcessor};
use carokia_planner::{Goal, Planner, RulePlanner};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

/// Configuration for the Brain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    /// Target tick rate in Hz.
    pub tick_rate_hz: f64,
    /// Short-term memory capacity.
    pub memory_capacity: usize,
    /// Conversation history limit.
    pub conversation_history_limit: usize,
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            tick_rate_hz: 10.0,
            memory_capacity: 100,
            conversation_history_limit: 50,
        }
    }
}

/// The main Brain orchestrator composing all subsystems.
pub struct Brain {
    pub config: BrainConfig,
    pub memory: ShortTermMemory,
    pub perception: PerceptionPipeline,
    pub conversation: ConversationManager,
    pub planner: RulePlanner,
    pub decision_engine: BehaviorDecisionEngine,
    pub goals: Vec<Goal>,
    pub tick_count: u64,
}

impl Brain {
    /// Create a new Brain with default subsystems.
    pub fn new(config: BrainConfig) -> Self {
        let memory = ShortTermMemory::new(config.memory_capacity);

        let mut perception = PerceptionPipeline::new();
        perception.add_processor(Box::new(StubProcessor::new(Modality::Vision)));
        perception.add_processor(Box::new(StubProcessor::new(Modality::Lidar)));
        perception.add_processor(Box::new(StubProcessor::new(Modality::Audio)));

        let conversation = ConversationManager::new(
            Box::new(MockBackend::new("Acknowledged.")),
            config.conversation_history_limit,
        );

        let planner = RulePlanner::new();
        let decision_engine = BehaviorDecisionEngine::with_defaults();

        Self {
            config,
            memory,
            perception,
            conversation,
            planner,
            decision_engine,
            goals: Vec::new(),
            tick_count: 0,
        }
    }

    /// Add a goal for the brain to pursue.
    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Run a single cognitive tick.
    pub async fn tick(&mut self) -> Result<Option<Action>, BrainError> {
        self.tick_count += 1;
        tracing::info!(tick = self.tick_count, "Brain tick");

        // 1. Simulate sensor input.
        let frame = SensorFrame::new(
            Modality::Lidar,
            SensorPayload::Json(serde_json::json!({"range": 5.0})),
        );

        // 2. Perception.
        let percepts = self.perception.process_frame(&frame).await?;
        tracing::debug!(count = percepts.len(), "Percepts generated");

        // 3. Store percepts in memory.
        for percept in &percepts {
            let entry = MemoryEntry::new(
                MemoryKind::Perception,
                format!("{:?}", percept.content),
                percept.confidence,
            );
            self.memory.store(entry).await?;
        }

        // 4. Plan: decompose any new goals.
        let mut all_tasks = Vec::new();
        for goal in &self.goals {
            let tasks = self.planner.decompose(goal).await?;
            all_tasks.extend(tasks);
        }

        // 5. Build world state.
        let memories = self
            .memory
            .recall(&carokia_memory::MemoryQuery::default())
            .await?;
        let world = WorldState {
            percepts,
            goals: self.goals.clone(),
            tasks: all_tasks,
            memories,
            tick: self.tick_count,
        };

        // 6. Decision.
        if let Some(cmd) = self.decision_engine.tick(&world).await? {
            tracing::info!(action = ?cmd.action, priority = ?cmd.priority, "Action selected");
            return Ok(Some(cmd.action));
        }

        Ok(None)
    }

    /// Run the main cognitive loop until cancelled.
    pub async fn run(&mut self, cancel: CancellationToken) -> Result<(), BrainError> {
        let tick_interval = Duration::from_secs_f64(1.0 / self.config.tick_rate_hz);
        tracing::info!(hz = self.config.tick_rate_hz, "Brain loop starting");

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    tracing::info!("Brain shutting down");
                    break;
                }
                _ = tokio::time::sleep(tick_interval) => {
                    match self.tick().await {
                        Ok(Some(action)) => {
                            tracing::info!(?action, "Executing action");
                        }
                        Ok(None) => {
                            tracing::debug!("No action this tick");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Tick error");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn brain_single_tick() {
        let config = BrainConfig::default();
        let mut brain = Brain::new(config);
        let action = brain.tick().await.unwrap();
        // Should produce an Idle halt at minimum.
        assert!(action.is_some());
    }

    #[tokio::test]
    async fn brain_with_goal_produces_task_action() {
        let config = BrainConfig::default();
        let mut brain = Brain::new(config);
        brain.add_goal(Goal::new("Say hello", 5));
        let action = brain.tick().await.unwrap();
        assert!(action.is_some());
    }

    #[tokio::test]
    async fn brain_run_cancellation() {
        let config = BrainConfig {
            tick_rate_hz: 100.0,
            ..Default::default()
        };
        let mut brain = Brain::new(config);
        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();

        let handle = tokio::spawn(async move {
            brain.run(cancel2).await
        });

        // Let it run a few ticks then cancel.
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel.cancel();

        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}
