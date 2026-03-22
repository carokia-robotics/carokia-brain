use async_trait::async_trait;
use carokia_core::BrainError;

/// Trait for generating vector embeddings from text.
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, BrainError>;
}

/// Ollama-based embedder that calls the Ollama embeddings API.
pub struct OllamaEmbedder {
    host: String,
    port: u16,
    model: String,
}

impl OllamaEmbedder {
    pub fn new(host: &str, port: u16, model: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl Embedder for OllamaEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, BrainError> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": self.model,
            "prompt": text
        });
        let resp = client
            .post(format!("{}:{}/api/embeddings", self.host, self.port))
            .json(&body)
            .send()
            .await
            .map_err(|e| BrainError::Internal(format!("Embedding request error: {e}")))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BrainError::Internal(format!("Embedding parse error: {e}")))?;

        let embedding: Vec<f32> = json["embedding"]
            .as_array()
            .ok_or_else(|| BrainError::Internal("No embedding in response".into()))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }
}

/// Mock embedder for testing. Produces deterministic fake embeddings based on text hash.
pub struct MockEmbedder;

#[async_trait]
impl Embedder for MockEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, BrainError> {
        let hash = text.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        Ok((0..384)
            .map(|i| ((hash.wrapping_mul(i + 1)) as f32 / u32::MAX as f32) - 0.5)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_embedder_deterministic() {
        let embedder = MockEmbedder;
        let emb1 = embedder.embed("hello world").await.unwrap();
        let emb2 = embedder.embed("hello world").await.unwrap();
        assert_eq!(emb1.len(), 384);
        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn mock_embedder_different_inputs() {
        let embedder = MockEmbedder;
        let emb1 = embedder.embed("hello world").await.unwrap();
        let emb2 = embedder.embed("goodbye world").await.unwrap();
        assert_ne!(emb1, emb2);
    }

    #[tokio::test]
    async fn mock_embedder_correct_dimension() {
        let embedder = MockEmbedder;
        let emb = embedder.embed("test").await.unwrap();
        assert_eq!(emb.len(), 384);
    }
}
