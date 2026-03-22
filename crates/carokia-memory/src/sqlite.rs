use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use carokia_core::BrainError;
use rusqlite::{params, Connection};

use crate::{cosine_similarity, MemoryEntry, MemoryKind, MemoryQuery, MemoryStore};

/// Persistent memory store backed by SQLite.
pub struct SqliteMemory {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteMemory {
    /// Open or create a SQLite database at `db_path`.
    /// Use ":memory:" for an in-memory database (great for tests).
    pub fn new(db_path: &str) -> Result<Self, BrainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| BrainError::Memory(format!("Failed to open SQLite DB: {e}")))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                timestamp_ms INTEGER NOT NULL,
                kind TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT NOT NULL,
                importance REAL NOT NULL,
                embedding BLOB
            );",
        )
        .map_err(|e| BrainError::Memory(format!("Failed to create table: {e}")))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Store an embedding for an existing memory entry.
    pub fn store_embedding(&self, id: &str, embedding: &[f32]) -> Result<(), BrainError> {
        let conn = self.conn.lock().map_err(|e| {
            BrainError::Memory(format!("Lock error: {e}"))
        })?;
        let blob = embedding_to_blob(embedding);
        conn.execute(
            "UPDATE memories SET embedding = ?1 WHERE id = ?2",
            params![blob, id],
        )
        .map_err(|e| BrainError::Memory(format!("Failed to store embedding: {e}")))?;
        Ok(())
    }
}

fn kind_to_str(kind: &MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Perception => "perception",
        MemoryKind::Conversation => "conversation",
        MemoryKind::Goal => "goal",
        MemoryKind::Event => "event",
        MemoryKind::Fact => "fact",
    }
}

fn str_to_kind(s: &str) -> MemoryKind {
    match s {
        "perception" => MemoryKind::Perception,
        "conversation" => MemoryKind::Conversation,
        "goal" => MemoryKind::Goal,
        "event" => MemoryKind::Event,
        "fact" => MemoryKind::Fact,
        _ => MemoryKind::Event,
    }
}

fn embedding_to_blob(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn blob_to_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

#[async_trait]
impl MemoryStore for SqliteMemory {
    async fn store(&mut self, entry: MemoryEntry) -> Result<(), BrainError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| {
                BrainError::Memory(format!("Lock error: {e}"))
            })?;
            let tags_json = serde_json::to_string(&entry.tags)
                .map_err(|e| BrainError::Memory(format!("Serialize tags: {e}")))?;
            conn.execute(
                "INSERT OR REPLACE INTO memories (id, timestamp_ms, kind, content, tags, importance, embedding) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    entry.id,
                    entry.timestamp.0 as i64,
                    kind_to_str(&entry.kind),
                    entry.content,
                    tags_json,
                    entry.importance,
                    Option::<Vec<u8>>::None,
                ],
            )
            .map_err(|e| BrainError::Memory(format!("Insert failed: {e}")))?;
            Ok(())
        })
        .await
        .map_err(|e| BrainError::Memory(format!("Task join error: {e}")))?
    }

    async fn recall(&self, query: &MemoryQuery) -> Result<Vec<MemoryEntry>, BrainError> {
        let conn = Arc::clone(&self.conn);
        let query = query.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| {
                BrainError::Memory(format!("Lock error: {e}"))
            })?;

            // If we have a query embedding, we need to load embeddings too
            let has_embedding_query = query.query_embedding.is_some();

            let mut sql = String::from(
                "SELECT id, timestamp_ms, kind, content, tags, importance, embedding FROM memories WHERE 1=1",
            );
            let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            if let Some(ref kind) = query.kind {
                sql.push_str(" AND kind = ?");
                params_vec.push(Box::new(kind_to_str(kind).to_string()));
            }

            if let Some(min_imp) = query.min_importance {
                sql.push_str(" AND importance >= ?");
                params_vec.push(Box::new(min_imp));
            }

            if !has_embedding_query {
                sql.push_str(" ORDER BY importance DESC, timestamp_ms DESC");
                if let Some(limit) = query.limit {
                    sql.push_str(&format!(" LIMIT {limit}"));
                }
            }

            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params_vec.iter().map(|b| b.as_ref()).collect();

            let mut stmt = conn.prepare(&sql).map_err(|e| {
                BrainError::Memory(format!("Prepare failed: {e}"))
            })?;

            let rows = stmt
                .query_map(param_refs.as_slice(), |row| {
                    let id: String = row.get(0)?;
                    let timestamp_ms: i64 = row.get(1)?;
                    let kind_str: String = row.get(2)?;
                    let content: String = row.get(3)?;
                    let tags_json: String = row.get(4)?;
                    let importance: f64 = row.get(5)?;
                    let embedding_blob: Option<Vec<u8>> = row.get(6)?;
                    Ok((id, timestamp_ms, kind_str, content, tags_json, importance, embedding_blob))
                })
                .map_err(|e| BrainError::Memory(format!("Query failed: {e}")))?;

            let mut entries = Vec::new();
            for row in rows {
                let (id, timestamp_ms, kind_str, content, tags_json, importance, embedding_blob) =
                    row.map_err(|e| BrainError::Memory(format!("Row error: {e}")))?;

                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

                // Filter by tag in Rust (SQLite JSON querying is more complex)
                if let Some(ref tag) = query.tag {
                    if !tags.contains(tag) {
                        continue;
                    }
                }

                let entry = MemoryEntry {
                    id,
                    timestamp: carokia_core::Timestamp(timestamp_ms as u64),
                    kind: str_to_kind(&kind_str),
                    content,
                    tags,
                    importance,
                };

                if has_embedding_query {
                    if let Some(ref query_emb) = query.query_embedding {
                        if let Some(ref blob) = embedding_blob {
                            let emb = blob_to_embedding(blob);
                            let sim = cosine_similarity(query_emb, &emb);
                            entries.push((entry, sim));
                        }
                        // Skip entries without embeddings when doing semantic search
                    } else {
                        entries.push((entry, 0.0));
                    }
                } else {
                    entries.push((entry, 0.0));
                }
            }

            if has_embedding_query {
                // Sort by similarity descending
                entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                if let Some(limit) = query.limit {
                    entries.truncate(limit);
                }
            }

            Ok(entries.into_iter().map(|(e, _)| e).collect())
        })
        .await
        .map_err(|e| BrainError::Memory(format!("Task join error: {e}")))?
    }

    async fn forget(&mut self, id: &str) -> Result<bool, BrainError> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| {
                BrainError::Memory(format!("Lock error: {e}"))
            })?;
            let deleted = conn
                .execute("DELETE FROM memories WHERE id = ?1", params![id])
                .map_err(|e| BrainError::Memory(format!("Delete failed: {e}")))?;
            Ok(deleted > 0)
        })
        .await
        .map_err(|e| BrainError::Memory(format!("Task join error: {e}")))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryKind;

    fn make_entry(kind: MemoryKind, content: &str, importance: f64) -> MemoryEntry {
        MemoryEntry::new(kind, content.to_string(), importance)
    }

    #[tokio::test]
    async fn sqlite_store_and_recall() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        let entry = make_entry(MemoryKind::Fact, "sky is blue", 0.8);
        mem.store(entry).await.unwrap();

        let results = mem.recall(&MemoryQuery::default()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "sky is blue");
    }

    #[tokio::test]
    async fn sqlite_forget() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        let entry = make_entry(MemoryKind::Event, "boom", 1.0);
        let id = entry.id.clone();
        mem.store(entry).await.unwrap();
        assert!(mem.forget(&id).await.unwrap());

        let results = mem.recall(&MemoryQuery::default()).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn sqlite_forget_nonexistent() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        assert!(!mem.forget("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn sqlite_filter_by_kind() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        mem.store(make_entry(MemoryKind::Fact, "fact1", 0.5))
            .await
            .unwrap();
        mem.store(make_entry(MemoryKind::Event, "event1", 0.5))
            .await
            .unwrap();

        let query = MemoryQuery {
            kind: Some(MemoryKind::Fact),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "fact1");
    }

    #[tokio::test]
    async fn sqlite_filter_by_tag() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        let entry = make_entry(MemoryKind::Fact, "tagged", 0.5)
            .with_tags(vec!["color".to_string()]);
        mem.store(entry).await.unwrap();
        mem.store(make_entry(MemoryKind::Fact, "untagged", 0.5))
            .await
            .unwrap();

        let query = MemoryQuery {
            tag: Some("color".to_string()),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "tagged");
    }

    #[tokio::test]
    async fn sqlite_filter_by_importance() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        mem.store(make_entry(MemoryKind::Fact, "low", 0.2))
            .await
            .unwrap();
        mem.store(make_entry(MemoryKind::Fact, "high", 0.9))
            .await
            .unwrap();

        let query = MemoryQuery {
            min_importance: Some(0.5),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "high");
    }

    #[tokio::test]
    async fn sqlite_limit() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();
        for i in 0..10 {
            mem.store(make_entry(MemoryKind::Fact, &format!("item{i}"), 0.5))
                .await
                .unwrap();
        }

        let query = MemoryQuery {
            limit: Some(3),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn sqlite_semantic_recall() {
        let mut mem = SqliteMemory::new(":memory:").unwrap();

        // Store two entries
        let entry1 = make_entry(MemoryKind::Fact, "The sky is blue", 0.8);
        let id1 = entry1.id.clone();
        mem.store(entry1).await.unwrap();

        let entry2 = make_entry(MemoryKind::Fact, "My owner is Bob", 0.8);
        let id2 = entry2.id.clone();
        mem.store(entry2).await.unwrap();

        // Assign embeddings: sky-related vector and person-related vector
        let sky_embedding = vec![1.0, 0.0, 0.0, 0.0];
        let person_embedding = vec![0.0, 1.0, 0.0, 0.0];

        mem.store_embedding(&id1, &sky_embedding).unwrap();
        mem.store_embedding(&id2, &person_embedding).unwrap();

        // Query with a vector similar to sky
        let query = MemoryQuery {
            query_embedding: Some(vec![0.9, 0.1, 0.0, 0.0]),
            limit: Some(1),
            ..Default::default()
        };
        let results = mem.recall(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "The sky is blue");
    }
}
