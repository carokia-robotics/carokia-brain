use async_trait::async_trait;
use carokia_core::BrainError;
use serde::{Deserialize, Serialize};

/// Parameters for LLM generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateParams {
    pub max_tokens: usize,
    pub temperature: f64,
    pub stop_sequences: Vec<String>,
}

impl Default for GenerateParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            stop_sequences: Vec::new(),
        }
    }
}

/// Backend trait for LLM inference.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn generate(&self, prompt: &str, params: &GenerateParams) -> Result<String, BrainError>;
}

/// A single turn in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,
    pub content: String,
}

/// Manages conversation history and interfaces with an LLM backend.
pub struct ConversationManager {
    history: Vec<ConversationTurn>,
    backend: Box<dyn LlmBackend>,
    max_history: usize,
}

impl ConversationManager {
    pub fn new(backend: Box<dyn LlmBackend>, max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            backend,
            max_history,
        }
    }

    pub fn history(&self) -> &[ConversationTurn] {
        &self.history
    }

    pub fn add_turn(&mut self, role: &str, content: &str) {
        self.history.push(ConversationTurn {
            role: role.to_string(),
            content: content.to_string(),
        });
        // Trim old turns.
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    pub async fn chat(&mut self, user_message: &str) -> Result<String, BrainError> {
        self.add_turn("user", user_message);

        // Build prompt from history.
        let prompt: String = self
            .history
            .iter()
            .map(|t| format!("{}: {}", t.role, t.content))
            .collect::<Vec<_>>()
            .join("\n");

        let response = self
            .backend
            .generate(&prompt, &GenerateParams::default())
            .await?;

        self.add_turn("assistant", &response);
        Ok(response)
    }
}

/// Mock backend that returns canned responses.
pub struct MockBackend {
    response: String,
}

impl MockBackend {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait]
impl LlmBackend for MockBackend {
    async fn generate(&self, _prompt: &str, _params: &GenerateParams) -> Result<String, BrainError> {
        Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_backend_returns_canned() {
        let backend = MockBackend::new("Hello, I am a robot.");
        let result = backend
            .generate("Hi", &GenerateParams::default())
            .await
            .unwrap();
        assert_eq!(result, "Hello, I am a robot.");
    }

    #[tokio::test]
    async fn conversation_manager_tracks_history() {
        let backend = MockBackend::new("Sure thing!");
        let mut mgr = ConversationManager::new(Box::new(backend), 20);
        let reply = mgr.chat("Do something").await.unwrap();
        assert_eq!(reply, "Sure thing!");
        assert_eq!(mgr.history().len(), 2); // user + assistant
    }

    #[tokio::test]
    async fn conversation_manager_trims_history() {
        let backend = MockBackend::new("ok");
        let mut mgr = ConversationManager::new(Box::new(backend), 4);
        for i in 0..5 {
            mgr.chat(&format!("msg {i}")).await.unwrap();
        }
        assert!(mgr.history().len() <= 4);
    }
}
