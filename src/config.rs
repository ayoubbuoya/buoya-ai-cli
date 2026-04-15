use serde::{Deserialize, Serialize};
use std::fs;

/// LLM Provider type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum LLMProvider {
    #[serde(rename = "ollama")]
    Ollama,
}

/// LLM model configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LLMConfig {
    /// LLM Provider type
    pub provider: LLMProvider,

    /// LLM API key
    #[serde(default)]
    pub api_key: String,

    /// LLM base url
    pub api_base_url: String,

    /// LLM model name (e.g., "gpt-4")
    /// This is used to specify which model to use when creating an agent.
    pub model: String,

    /// Optional system instruction preamble for the agent.
    #[serde(default)]
    pub system_instruction: String,

    /// Temperature setting for the LLM (e.g., 0.7)
    pub temperature: f64,

    /// think mode
    pub think: bool,
}

impl LLMConfig {
    /// Load configuration from config.toml file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: LLMConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
