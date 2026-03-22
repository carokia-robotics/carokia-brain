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

#[cfg(feature = "streaming")]
#[async_trait]
impl crate::StreamingLlmBackend for OllamaBackend {
    async fn generate_stream(
        &self,
        prompt: &str,
        _params: &GenerateParams,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String, BrainError>>, BrainError> {
        use ollama_rs::generation::completion::request::GenerationRequest;
        use tokio_stream::StreamExt;

        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());

        let mut stream = self
            .client
            .generate_stream(request)
            .await
            .map_err(|e| BrainError::Language(format!("Ollama stream error: {e}")))?;

        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunks) => {
                        for chunk in chunks {
                            if tx.send(Ok(chunk.response)).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(Err(BrainError::Language(format!("Ollama stream: {e}"))))
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
