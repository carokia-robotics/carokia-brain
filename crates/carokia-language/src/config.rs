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

/// Personality configuration for the AI companion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityConfig {
    pub name: String,
    pub traits: Vec<String>,
    pub speaking_style: String,
    pub backstory: String,
    /// One of: "concise", "moderate", "detailed"
    pub response_length: String,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            name: "Carokia".to_string(),
            traits: vec![
                "helpful".to_string(),
                "protective".to_string(),
                "loyal".to_string(),
                "curious".to_string(),
            ],
            speaking_style: "warm and concise".to_string(),
            backstory: "An advanced autonomous robot companion, designed to assist and protect."
                .to_string(),
            response_length: "concise".to_string(),
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

    #[test]
    fn personality_config_default() {
        let config = PersonalityConfig::default();
        assert_eq!(config.name, "Carokia");
        assert!(config.traits.contains(&"helpful".to_string()));
        assert_eq!(config.response_length, "concise");
    }

    #[test]
    fn personality_config_deserialize() {
        let toml_str = r#"
            name = "Buddy"
            traits = ["friendly", "witty"]
            speaking_style = "informal"
            backstory = "A digital friend."
            response_length = "detailed"
        "#;
        let config: PersonalityConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.name, "Buddy");
        assert_eq!(config.traits, vec!["friendly", "witty"]);
        assert_eq!(config.response_length, "detailed");
    }
}
