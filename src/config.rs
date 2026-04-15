use serde::{Deserialize, Serialize};

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
}
