use anyhow::Result;
use rig::{agent::Agent, client::CompletionClient, completion::Prompt};

use crate::config::{LLMConfig, LLMProvider};

/// Unified Agent enum
pub enum ProviderAgent {
    Ollama(Agent<rig::providers::ollama::CompletionModel>),
}

impl ProviderAgent {
    pub fn new(llm_config: &LLMConfig) -> Result<Self> {
        match llm_config.provider {
            LLMProvider::Ollama => {
                let client = rig::providers::ollama::Client::builder()
                    .api_key(rig::client::Nothing)
                    .base_url(&llm_config.api_base_url)
                    .build()?;

                let agent = client
                    .agent(&llm_config.model)
                    .preamble(&llm_config.system_instruction)
                    .temperature(llm_config.temperature)
                    .build();

                Ok(ProviderAgent::Ollama(agent))
            }
        }
    }

    pub async fn prompt(&self, prompt: &str) -> Result<String> {
        match self {
            ProviderAgent::Ollama(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
        }
    }
}
