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

#[cfg(feature = "streaming")]
#[async_trait]
impl crate::StreamingLlmBackend for ClaudeBackend {
    async fn generate_stream(
        &self,
        prompt: &str,
        params: &GenerateParams,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String, BrainError>>, BrainError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": params.max_tokens,
            "stream": true,
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
            .map_err(|e| BrainError::Language(format!("Claude stream HTTP error: {e}")))?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            use tokio_stream::StreamExt;

            let mut stream = resp.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete SSE lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            let line = line.trim();
                            if line.is_empty() {
                                continue;
                            }

                            // Parse SSE data lines
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    return;
                                }
                                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                    // content_block_delta events have delta.text
                                    if json["type"] == "content_block_delta" {
                                        if let Some(text) = json["delta"]["text"].as_str() {
                                            if tx.send(Ok(text.to_string())).await.is_err() {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(BrainError::Language(format!(
                                "Claude stream error: {e}"
                            ))))
                            .await;
                        return;
                    }
                }
            }
        });

        Ok(rx)
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
