mod config;
mod llm;

use anyhow::Result;

use crate::config::LLMConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let config = LLMConfig {
        provider: config::LLMProvider::Ollama,
        api_key: "".to_string(),
        api_base_url: "http://localhost:11434".to_string(),
        model: "qwen3.5:0.8b ".to_string(),
        system_instruction: "You are a helpful assistant.".to_string(),
    };

    let agent = llm::agent::LLMAgent::new(config)?;

    let prompt = "What is the capital of France?";

    let response = agent.prompt(prompt).await?;

    println!("Response: {}", response);

    Ok(())
}
