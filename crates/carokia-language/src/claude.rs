use async_trait::async_trait;
use carokia_core::BrainError;

use crate::{GenerateParams, LlmBackend};

pub struct ClaudeBackend {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeBackend {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmBackend for ClaudeBackend {
    async fn generate(&self, prompt: &str, params: &GenerateParams) -> Result<String, BrainError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": params.max_tokens,
            "messages": [{"role": "user", "content": prompt}]
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BrainError::Language(format!("Claude HTTP error: {e}")))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BrainError::Language(format!("Claude parse error: {e}")))?;

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| BrainError::Language("No text in Claude response".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_backend_creates_successfully() {
        let backend = ClaudeBackend::new("sk-test", "claude-sonnet-4-20250514");
        assert_eq!(backend.model, "claude-sonnet-4-20250514");
    }

    #[tokio::test]
    #[ignore] // Requires valid API key
    async fn claude_backend_generates_response() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let backend = ClaudeBackend::new(&api_key, "claude-sonnet-4-20250514");
        let result = backend
            .generate("Say hello in one word.", &GenerateParams::default())
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
