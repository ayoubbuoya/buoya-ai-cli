use anyhow::Result;
use rig::tool::ToolDyn;

use crate::config::LLMConfig;
use crate::llm::agent::providers::ProviderAgent;

pub mod providers;

pub struct LLMAgent {
    agent: ProviderAgent,
    config: LLMConfig,
}

impl LLMAgent {
    pub fn new(config: LLMConfig, tools: Vec<Box<dyn ToolDyn>>) -> Result<Self> {
        let agent = ProviderAgent::new(&config, tools)?;
        Ok(LLMAgent { agent, config })
    }

    pub async fn prompt(&self, prompt: &str) -> Result<String> {
        self.agent.prompt(prompt).await
    }
}

/* pub struct LLMAgentBuilder {
    config: LLMConfig,
    tools: Vec<Box<dyn ToolDyn>>,
}

impl LLMAgentBuilder {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            tools: Vec::new(),
        }
    }

    pub fn tool(mut self, tool: impl ToolDyn + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    pub fn tools(mut self, tools: Vec<Box<dyn ToolDyn>>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn build(self) -> Result<LLMAgent> {
        LLMAgent::new(self.config, self.tools)
    }
}
 */
