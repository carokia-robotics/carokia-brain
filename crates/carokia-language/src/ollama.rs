use async_trait::async_trait;
use carokia_core::BrainError;
use ollama_rs::Ollama;

use crate::{GenerateParams, LlmBackend};

pub struct OllamaBackend {
    client: Ollama,
    model: String,
}

impl OllamaBackend {
    pub fn new(host: &str, port: u16, model: &str) -> Self {
        let client = Ollama::new(host, port);
        Self {
            client,
            model: model.to_string(),
        }
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    async fn generate(&self, prompt: &str, _params: &GenerateParams) -> Result<String, BrainError> {
        use ollama_rs::generation::completion::request::GenerationRequest;

        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());

        let response = self
            .client
            .generate(request)
            .await
            .map_err(|e| BrainError::Language(format!("Ollama error: {e}")))?;

        Ok(response.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_backend_creates_successfully() {
        let backend = OllamaBackend::new("http://localhost", 11434, "llama3.2");
        assert_eq!(backend.model, "llama3.2");
    }

    #[tokio::test]
    #[ignore] // Requires running Ollama instance
    async fn ollama_backend_generates_response() {
        let backend = OllamaBackend::new("http://localhost", 11434, "llama3.2");
        let result = backend
            .generate("Say hello in one word.", &GenerateParams::default())
            .await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }
}
