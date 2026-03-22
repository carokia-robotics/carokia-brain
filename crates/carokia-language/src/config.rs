use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum LlmProviderConfig {
    #[serde(rename = "ollama")]
    Ollama {
        #[serde(default = "default_ollama_host")]
        host: String,
        #[serde(default = "default_ollama_port")]
        port: u16,
        #[serde(default = "default_ollama_model")]
        model: String,
    },
    #[serde(rename = "claude")]
    Claude {
        api_key: String,
        #[serde(default = "default_claude_model")]
        model: String,
    },
    #[serde(rename = "mock")]
    Mock {
        #[serde(default = "default_mock_response")]
        response: String,
    },
}

fn default_ollama_host() -> String {
    "http://localhost".to_string()
}
fn default_ollama_port() -> u16 {
    11434
}
fn default_ollama_model() -> String {
    "gemma3:latest".to_string()
}
fn default_claude_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}
fn default_mock_response() -> String {
    "Mock response".to_string()
}

impl Default for LlmProviderConfig {
    fn default() -> Self {
        Self::Ollama {
            host: default_ollama_host(),
            port: default_ollama_port(),
            model: default_ollama_model(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_ollama_config_from_toml() {
        let toml_str = r#"
            provider = "ollama"
            host = "http://myhost"
            port = 9999
            model = "mistral"
        "#;
        let config: LlmProviderConfig = toml::from_str(toml_str).unwrap();
        match config {
            LlmProviderConfig::Ollama { host, port, model } => {
                assert_eq!(host, "http://myhost");
                assert_eq!(port, 9999);
                assert_eq!(model, "mistral");
            }
            _ => panic!("Expected Ollama variant"),
        }
    }

    #[test]
    fn deserialize_ollama_config_defaults() {
        let toml_str = r#"provider = "ollama""#;
        let config: LlmProviderConfig = toml::from_str(toml_str).unwrap();
        match config {
            LlmProviderConfig::Ollama { host, port, model } => {
                assert_eq!(host, "http://localhost");
                assert_eq!(port, 11434);
                assert_eq!(model, "gemma3:latest");
            }
            _ => panic!("Expected Ollama variant"),
        }
    }

    #[test]
    fn deserialize_claude_config_from_toml() {
        let toml_str = r#"
            provider = "claude"
            api_key = "sk-test-123"
        "#;
        let config: LlmProviderConfig = toml::from_str(toml_str).unwrap();
        match config {
            LlmProviderConfig::Claude { api_key, model } => {
                assert_eq!(api_key, "sk-test-123");
                assert_eq!(model, "claude-sonnet-4-20250514");
            }
            _ => panic!("Expected Claude variant"),
        }
    }

    #[test]
    fn deserialize_mock_config_from_toml() {
        let toml_str = r#"provider = "mock""#;
        let config: LlmProviderConfig = toml::from_str(toml_str).unwrap();
        match config {
            LlmProviderConfig::Mock { response } => {
                assert_eq!(response, "Mock response");
            }
            _ => panic!("Expected Mock variant"),
        }
    }

    #[test]
    fn default_config_is_ollama() {
        let config = LlmProviderConfig::default();
        matches!(config, LlmProviderConfig::Ollama { .. });
    }
}
