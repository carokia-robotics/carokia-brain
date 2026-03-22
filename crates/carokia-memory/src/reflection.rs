use std::sync::Arc;

use carokia_core::BrainError;
use carokia_language::{GenerateParams, LlmBackend};

use crate::{MemoryEntry, MemoryKind};

/// Engine for self-reflection over recent memories.
///
/// Periodically analyzes recent memory entries using an LLM to extract
/// insights and patterns, storing them as new semantic memory entries.
pub struct ReflectionEngine {
    backend: Arc<dyn LlmBackend>,
    reflection_interval: usize,
}

impl ReflectionEngine {
    /// Create a new reflection engine.
    ///
    /// * `backend` - The LLM backend to use for generating reflections.
    /// * `interval` - Reflect every N memory entries.
    pub fn new(backend: Arc<dyn LlmBackend>, interval: usize) -> Self {
        Self {
            backend,
            reflection_interval: interval,
        }
    }

    /// Reflect on recent memories and produce an insight as a new memory entry.
    pub async fn reflect(
        &self,
        recent_memories: &[MemoryEntry],
    ) -> Result<MemoryEntry, BrainError> {
        let mut prompt = String::from(
            "You are an introspective AI agent. Review the following recent memories \
             and provide a brief insight or pattern you notice. \
             Be concise (1-2 sentences).\n\nRecent memories:\n",
        );

        for (i, memory) in recent_memories.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. [{:?}] {}\n",
                i + 1,
                memory.kind,
                memory.content
            ));
        }

        prompt.push_str("\nInsight:");

        let params = GenerateParams {
            max_tokens: 256,
            temperature: 0.5,
            stop_sequences: vec![],
        };

        let response = self.backend.generate(&prompt, &params).await?;

        let entry = MemoryEntry::new(MemoryKind::Fact, response.trim().to_string(), 0.8)
            .with_tags(vec!["reflection".to_string(), "insight".to_string()]);

        Ok(entry)
    }

    /// Check if it's time to reflect based on the current memory count.
    pub fn should_reflect(&self, memory_count: usize) -> bool {
        memory_count > 0 && memory_count.is_multiple_of(self.reflection_interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_language::MockBackend;

    #[test]
    fn should_reflect_at_interval() {
        let backend = Arc::new(MockBackend::new("insight"));
        let engine = ReflectionEngine::new(backend, 5);

        assert!(!engine.should_reflect(0));
        assert!(!engine.should_reflect(1));
        assert!(!engine.should_reflect(3));
        assert!(engine.should_reflect(5));
        assert!(engine.should_reflect(10));
        assert!(engine.should_reflect(15));
        assert!(!engine.should_reflect(7));
    }

    #[test]
    fn should_reflect_interval_one() {
        let backend = Arc::new(MockBackend::new("insight"));
        let engine = ReflectionEngine::new(backend, 1);

        assert!(!engine.should_reflect(0));
        assert!(engine.should_reflect(1));
        assert!(engine.should_reflect(2));
        assert!(engine.should_reflect(100));
    }

    #[tokio::test]
    async fn reflect_produces_memory_entry() {
        let backend = Arc::new(MockBackend::new(
            "I notice a pattern of repeated obstacle detections.",
        ));
        let engine = ReflectionEngine::new(backend, 5);

        let memories = vec![
            MemoryEntry::new(MemoryKind::Perception, "Obstacle at 2m".into(), 0.5),
            MemoryEntry::new(MemoryKind::Perception, "Obstacle at 1.5m".into(), 0.6),
            MemoryEntry::new(MemoryKind::Event, "Avoided obstacle".into(), 0.7),
        ];

        let insight = engine.reflect(&memories).await.unwrap();
        assert_eq!(insight.kind, MemoryKind::Fact);
        assert!(insight.content.contains("pattern"));
        assert!(insight.tags.contains(&"reflection".to_string()));
        assert!(insight.tags.contains(&"insight".to_string()));
        assert_eq!(insight.importance, 0.8);
    }

    #[tokio::test]
    async fn reflect_on_empty_memories() {
        let backend = Arc::new(MockBackend::new("No memories to reflect on."));
        let engine = ReflectionEngine::new(backend, 5);

        let insight = engine.reflect(&[]).await.unwrap();
        assert_eq!(insight.kind, MemoryKind::Fact);
        assert!(!insight.content.is_empty());
    }
}
