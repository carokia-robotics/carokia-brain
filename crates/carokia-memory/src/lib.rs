use std::collections::VecDeque;

use async_trait::async_trait;
use carokia_core::{BrainError, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kind of memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    Perception,
    Conversation,
    Goal,
    Event,
    Fact,
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub timestamp: Timestamp,
    pub kind: MemoryKind,
    pub content: String,
    pub tags: Vec<String>,
    pub importance: f64,
}

impl MemoryEntry {
    pub fn new(kind: MemoryKind, content: String, importance: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            kind,
            content,
            tags: Vec::new(),
            importance,
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Query for filtering memories.
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub kind: Option<MemoryKind>,
    pub tag: Option<String>,
    pub min_importance: Option<f64>,
    pub limit: Option<usize>,
    /// Optional query embedding vector for semantic recall.
    pub query_embedding: Option<Vec<f32>>,
}

/// Compute cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "embeddings")]
pub mod embeddings;

#[cfg(feature = "reflection")]
pub mod reflection;

/// Trait for memory storage backends.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store(&mut self, entry: MemoryEntry) -> Result<(), BrainError>;
    async fn recall(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>, BrainError>;
    async fn forget(&mut self, id: &str) -> Result<bool, BrainError>;
}

/// Bounded short-term memory backed by a VecDeque.
pub struct ShortTermMemory {
    capacity: usize,
    entries: VecDeque<MemoryEntry>,
}

impl ShortTermMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[async_trait]
impl MemoryStore for ShortTermMemory {
    async fn store(&mut self, entry: MemoryEntry) -> Result<(), BrainError> {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
        Ok(())
    }

    async fn recall(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>, BrainError> {
        let mut results: Vec<MemoryEntry> = self
            .entries
            .iter()
            .filter(|e| {
                if let Some(ref kind) = query.kind {
                    if &e.kind != kind {
                        return false;
                    }
                }
                if let Some(ref tag) = query.tag {
                    if !e.tags.contains(tag) {
                        return false;
                    }
                }
                if let Some(min_imp) = query.min_importance {
                    if e.importance < min_imp {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Most recent first.
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn forget(&mut self, id: &str) -> Result<bool, BrainError> {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        Ok(self.entries.len() < before)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_and_recall() {
        let mut mem = ShortTermMemory::new(10);
        let entry = MemoryEntry::new(MemoryKind::Fact, "sky is blue".into(), 0.5);
        mem.store(entry).await.unwrap();
        let results = mem.recall(&MemoryQuery::default()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "sky is blue");
    }

    #[tokio::test]
    async fn capacity_limit_evicts_oldest() {
        let mut mem = ShortTermMemory::new(2);
        mem.store(MemoryEntry::new(MemoryKind::Fact, "a".into(), 0.1))
            .await
            .unwrap();
        mem.store(MemoryEntry::new(MemoryKind::Fact, "b".into(), 0.2))
            .await
            .unwrap();
        mem.store(MemoryEntry::new(MemoryKind::Fact, "c".into(), 0.3))
            .await
            .unwrap();
        assert_eq!(mem.len(), 2);
        let results = mem.recall(&MemoryQuery::default()).await.unwrap();
        assert!(results.iter().all(|e| e.content != "a"));
    }

    #[tokio::test]
    async fn forget_removes_entry() {
        let mut mem = ShortTermMemory::new(10);
        let entry = MemoryEntry::new(MemoryKind::Event, "boom".into(), 1.0);
        let id = entry.id.clone();
        mem.store(entry).await.unwrap();
        assert!(mem.forget(&id).await.unwrap());
        assert!(mem.is_empty());
    }

    #[test]
    fn cosine_similarity_identical_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &a);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_orthogonal_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_opposite_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[tokio::test]
    async fn query_by_kind() {
        let mut mem = ShortTermMemory::new(10);
        mem.store(MemoryEntry::new(MemoryKind::Fact, "fact1".into(), 0.5))
            .await
            .unwrap();
        mem.store(MemoryEntry::new(MemoryKind::Event, "event1".into(), 0.5))
            .await
            .unwrap();
        let query = MemoryQuery {
            kind: Some(MemoryKind::Fact),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, MemoryKind::Fact);
    }
}
