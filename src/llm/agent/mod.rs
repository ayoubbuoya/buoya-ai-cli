use anyhow::Result;

use crate::{config::LLMConfig, llm::agent::providers::ProviderAgent};

pub mod providers;

pub struct LLMAgent {
    agent: ProviderAgent,
    config: LLMConfig,
}

impl LLMAgent {
    pub fn new(config: LLMConfig) -> Result<Self> {
        let agent = ProviderAgent::new(&config)?;
        Ok(LLMAgent { agent, config })
    }


    pub async fn prompt(&self, prompt: &str) -> Result<String> {
        self.agent.prompt(prompt).await
    }
}
