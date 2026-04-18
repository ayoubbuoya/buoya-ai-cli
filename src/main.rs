mod config;
mod llm;
mod types;

use anyhow::Result;

use crate::config::LLMConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let config = LLMConfig::from_file("config.toml")?;

    let agent = llm::agent::LLMAgent::new(config)?;

    let prompt = "What is the capital of Tunisia?";

    let response = agent.prompt(prompt).await?;

    println!("Response: {}", response);

    Ok(())
}
