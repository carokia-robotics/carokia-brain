//! LLM-powered vision analysis using Ollama multimodal models.
//!
//! Uses a vision-capable model (e.g. llava) via the Ollama API to describe
//! scenes, detect objects, and understand visual context. This is more capable
//! than traditional YOLO for general scene understanding and requires no
//! additional ML model files beyond `ollama pull llava`.

use base64::Engine;
use carokia_core::BrainError;
use serde::{Deserialize, Serialize};

/// Result of analyzing an image with the vision model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    /// Free-text description of the scene.
    pub description: String,
    /// Objects detected in the scene (extracted from the description).
    pub objects: Vec<String>,
    /// Timestamp when analysis was performed (millis since epoch).
    pub timestamp_ms: u64,
}

/// Configuration for the vision analyzer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub ollama_host: String,
    pub ollama_port: u16,
    pub model: String,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost".to_string(),
            ollama_port: 11434,
            model: "llava".to_string(),
        }
    }
}

/// Analyzes images using an Ollama vision model (e.g. llava).
pub struct VisionAnalyzer {
    config: VisionConfig,
    client: reqwest::Client,
}

impl VisionAnalyzer {
    pub fn new(config: VisionConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// The Ollama model name used for vision analysis.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// The full Ollama API base URL.
    fn api_url(&self) -> String {
        format!("{}:{}", self.config.ollama_host, self.config.ollama_port)
    }

    /// Encode image bytes as base64.
    pub fn encode_image(image_bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(image_bytes)
    }

    /// Analyze an image and return a scene description with detected objects.
    pub async fn analyze(&self, image_bytes: &[u8]) -> Result<VisionResult, BrainError> {
        let b64 = Self::encode_image(image_bytes);

        let body = serde_json::json!({
            "model": self.config.model,
            "prompt": "Describe what you see in this image concisely. List any objects, people, or notable features as a comma-separated list after 'Objects:' on a new line.",
            "images": [b64],
            "stream": false,
        });

        let resp = self
            .client
            .post(format!("{}/api/generate", self.api_url()))
            .json(&body)
            .send()
            .await
            .map_err(|e| BrainError::Perception(format!("vision API request error: {e}")))?;

        if !resp.status().is_success() {
            return Err(BrainError::Perception(format!(
                "vision API returned status {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BrainError::Perception(format!("vision API parse error: {e}")))?;

        let description = json["response"].as_str().unwrap_or("").to_string();

        let objects = extract_objects(&description);

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(VisionResult {
            description,
            objects,
            timestamp_ms,
        })
    }

    /// Analyze an image with a custom prompt.
    pub async fn analyze_with_prompt(
        &self,
        image_bytes: &[u8],
        prompt: &str,
    ) -> Result<String, BrainError> {
        let b64 = Self::encode_image(image_bytes);

        let body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "images": [b64],
            "stream": false,
        });

        let resp = self
            .client
            .post(format!("{}/api/generate", self.api_url()))
            .json(&body)
            .send()
            .await
            .map_err(|e| BrainError::Perception(format!("vision API request error: {e}")))?;

        if !resp.status().is_success() {
            return Err(BrainError::Perception(format!(
                "vision API returned status {}",
                resp.status()
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BrainError::Perception(format!("vision API parse error: {e}")))?;

        Ok(json["response"].as_str().unwrap_or("").to_string())
    }
}

/// Extract object names from a vision model response.
///
/// Looks for a line starting with "Objects:" and splits by comma.
/// Falls back to an empty list if no such line is found.
fn extract_objects(description: &str) -> Vec<String> {
    for line in description.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Objects:") {
            return rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_image_produces_base64() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let encoded = VisionAnalyzer::encode_image(&data);
        assert!(!encoded.is_empty());
        // Verify round-trip
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn extract_objects_from_description() {
        let desc = "A room with furniture.\nObjects: chair, table, lamp, window";
        let objects = extract_objects(desc);
        assert_eq!(objects, vec!["chair", "table", "lamp", "window"]);
    }

    #[test]
    fn extract_objects_empty_when_no_prefix() {
        let desc = "Just a plain description with no structured output.";
        let objects = extract_objects(desc);
        assert!(objects.is_empty());
    }

    #[test]
    fn vision_result_construction() {
        let result = VisionResult {
            description: "A cat on a table".to_string(),
            objects: vec!["cat".to_string(), "table".to_string()],
            timestamp_ms: 1234567890,
        };
        assert_eq!(result.objects.len(), 2);
        assert_eq!(result.description, "A cat on a table");
    }

    #[test]
    fn vision_config_defaults() {
        let config = VisionConfig::default();
        assert_eq!(config.ollama_host, "http://localhost");
        assert_eq!(config.ollama_port, 11434);
        assert_eq!(config.model, "llava");
    }
}
