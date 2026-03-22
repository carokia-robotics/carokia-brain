use async_trait::async_trait;
use carokia_core::BrainError;
use serde::{Deserialize, Serialize};

pub mod config;

#[cfg(feature = "ollama")]
pub mod ollama;

#[cfg(feature = "claude")]
pub mod claude;

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
    system_prompt: Option<String>,
}

impl ConversationManager {
    pub fn new(backend: Box<dyn LlmBackend>, max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            backend,
            max_history,
            system_prompt: None,
        }
    }

    pub fn with_system_prompt(
        backend: Box<dyn LlmBackend>,
        max_history: usize,
        system_prompt: String,
    ) -> Self {
        Self {
            history: Vec::new(),
            backend,
            max_history,
            system_prompt: Some(system_prompt),
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

        // Build prompt from system prompt + history.
        let mut parts = Vec::new();

        if let Some(ref system_prompt) = self.system_prompt {
            parts.push(format!("system: {system_prompt}"));
        }

        for t in &self.history {
            parts.push(format!("{}: {}", t.role, t.content));
        }

        let prompt = parts.join("\n");

        let response = self
            .backend
            .generate(&prompt, &GenerateParams::default())
            .await?;

        self.add_turn("assistant", &response);
        Ok(response)
    }
}

/// Create an LLM backend from a provider configuration.
pub fn create_backend(config: &config::LlmProviderConfig) -> Box<dyn LlmBackend> {
    match config {
        #[cfg(feature = "ollama")]
        config::LlmProviderConfig::Ollama { host, port, model } => {
            Box::new(ollama::OllamaBackend::new(host, *port, model))
        }
        #[cfg(not(feature = "ollama"))]
        config::LlmProviderConfig::Ollama { .. } => {
            panic!("Ollama feature is not enabled. Compile with --features ollama")
        }
        #[cfg(feature = "claude")]
        config::LlmProviderConfig::Claude { api_key, model } => {
            Box::new(claude::ClaudeBackend::new(api_key, model))
        }
        #[cfg(not(feature = "claude"))]
        config::LlmProviderConfig::Claude { .. } => {
            panic!("Claude feature is not enabled. Compile with --features claude")
        }
        config::LlmProviderConfig::Mock { response } => {
            Box::new(MockBackend::new(response.clone()))
        }
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
    async fn generate(
        &self,
        _prompt: &str,
        _params: &GenerateParams,
    ) -> Result<String, BrainError> {
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

    #[tokio::test]
    async fn conversation_manager_with_system_prompt() {
        let backend = MockBackend::new("I am Carokia!");
        let mut mgr = ConversationManager::with_system_prompt(
            Box::new(backend),
            20,
            "You are a helpful robot.".to_string(),
        );
        let reply = mgr.chat("Who are you?").await.unwrap();
        assert_eq!(reply, "I am Carokia!");
        assert_eq!(mgr.history().len(), 2);
    }

    #[tokio::test]
    async fn conversation_manager_without_system_prompt_has_none() {
        let backend = MockBackend::new("ok");
        let mgr = ConversationManager::new(Box::new(backend), 20);
        assert!(mgr.system_prompt.is_none());
    }

    #[test]
    fn create_backend_mock() {
        let config = config::LlmProviderConfig::Mock {
            response: "test".to_string(),
        };
        let _backend = create_backend(&config);
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn create_backend_ollama() {
        let config = config::LlmProviderConfig::default();
        let _backend = create_backend(&config);
    }
}
