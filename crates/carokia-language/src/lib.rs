use async_trait::async_trait;
use carokia_core::BrainError;
use serde::{Deserialize, Serialize};

pub mod config;
pub mod tools;
pub mod tts;

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

/// Streaming backend trait for LLM inference.
#[cfg(feature = "streaming")]
#[async_trait]
pub trait StreamingLlmBackend: LlmBackend {
    async fn generate_stream(
        &self,
        prompt: &str,
        params: &GenerateParams,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String, BrainError>>, BrainError>;
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
    context: Option<String>,
}

impl ConversationManager {
    pub fn new(backend: Box<dyn LlmBackend>, max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            backend,
            max_history,
            system_prompt: None,
            context: None,
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
            context: None,
        }
    }

    /// Set extra context that will be prepended to the prompt (e.g. memory).
    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }

    /// Clear the context.
    pub fn clear_context(&mut self) {
        self.context = None;
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

    /// Build the full prompt from system prompt, context, and history.
    fn build_prompt(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref system_prompt) = self.system_prompt {
            parts.push(format!("system: {system_prompt}"));
        }

        if let Some(ref context) = self.context {
            parts.push(format!("context: {context}"));
        }

        for t in &self.history {
            parts.push(format!("{}: {}", t.role, t.content));
        }

        parts.join("\n")
    }

    pub async fn chat(&mut self, user_message: &str) -> Result<String, BrainError> {
        self.add_turn("user", user_message);

        let prompt = self.build_prompt();

        let response = self
            .backend
            .generate(&prompt, &GenerateParams::default())
            .await?;

        self.add_turn("assistant", &response);
        Ok(response)
    }
}

/// Build a system prompt from a PersonalityConfig.
pub fn build_personality_prompt(personality: &config::PersonalityConfig) -> String {
    let mut parts = Vec::new();
    parts.push(format!("You are {}, an AI companion.", personality.name));
    if !personality.traits.is_empty() {
        parts.push(format!(
            "Your personality traits: {}.",
            personality.traits.join(", ")
        ));
    }
    if !personality.speaking_style.is_empty() {
        parts.push(format!("Speaking style: {}.", personality.speaking_style));
    }
    if !personality.backstory.is_empty() {
        parts.push(format!("Backstory: {}", personality.backstory));
    }
    parts.push(format!("Keep responses {}.", personality.response_length));
    parts.join(" ")
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

#[cfg(feature = "streaming")]
#[async_trait]
impl StreamingLlmBackend for MockBackend {
    async fn generate_stream(
        &self,
        _prompt: &str,
        _params: &GenerateParams,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String, BrainError>>, BrainError> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let words: Vec<String> = self
            .response
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();
        tokio::spawn(async move {
            for (i, word) in words.into_iter().enumerate() {
                let token = if i == 0 { word } else { format!(" {word}") };
                if tx.send(Ok(token)).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
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

    #[tokio::test]
    async fn conversation_manager_with_context() {
        let backend = MockBackend::new("I remember!");
        let mut mgr = ConversationManager::with_system_prompt(
            Box::new(backend),
            20,
            "You are helpful.".to_string(),
        );
        mgr.set_context("Memory: The user likes cats.".to_string());
        let reply = mgr.chat("What do I like?").await.unwrap();
        assert_eq!(reply, "I remember!");
        // Context should be included in the prompt
        let prompt = mgr.build_prompt();
        assert!(prompt.contains("Memory: The user likes cats."));
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

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn mock_streaming_yields_tokens() {
        let backend = MockBackend::new("hello world foo bar");
        let mut rx = backend
            .generate_stream("test", &GenerateParams::default())
            .await
            .unwrap();

        let mut tokens = Vec::new();
        while let Some(result) = rx.recv().await {
            tokens.push(result.unwrap());
        }
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], "hello");
        assert_eq!(tokens[1], " world");
        assert_eq!(tokens[2], " foo");
        assert_eq!(tokens[3], " bar");
        // Reassembled should match original
        let reassembled: String = tokens.into_iter().collect();
        assert_eq!(reassembled, "hello world foo bar");
    }

    #[test]
    fn build_personality_prompt_includes_traits() {
        let personality = config::PersonalityConfig {
            name: "Carokia".to_string(),
            traits: vec!["helpful".to_string(), "curious".to_string()],
            speaking_style: "warm and friendly".to_string(),
            backstory: "An AI companion.".to_string(),
            response_length: "concise".to_string(),
        };
        let prompt = build_personality_prompt(&personality);
        assert!(prompt.contains("Carokia"));
        assert!(prompt.contains("helpful"));
        assert!(prompt.contains("curious"));
        assert!(prompt.contains("warm and friendly"));
        assert!(prompt.contains("concise"));
    }

    #[test]
    fn personality_config_deserializes_from_toml() {
        let toml_str = r#"
            name = "TestBot"
            traits = ["brave", "smart"]
            speaking_style = "casual"
            backstory = "A test robot."
            response_length = "moderate"
        "#;
        let config: config::PersonalityConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.name, "TestBot");
        assert_eq!(config.traits, vec!["brave", "smart"]);
        assert_eq!(config.speaking_style, "casual");
        assert_eq!(config.response_length, "moderate");
    }
}
