mod config;
mod llm;
mod types;

use anyhow::Result;

use crate::config::LLMConfig;
use crate::llm::tools::file_explorer::AgentToolFileExplorer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = LLMConfig::from_file("./src/config/config.toml")?;

    println!("Config: {:?}", config);

    let project_path = std::env::current_dir()?;

    println!("Project path: {}", project_path.display());

    let agent = llm::agent::LLMAgent::new(
        config,
        vec![Box::new(AgentToolFileExplorer::new(project_path))],
    )?;

    let prompt = "Explore the current project directory structure. List the top-level files and directories, then tell me what kind of project this is based on the files you see.";

    let response = agent.prompt(prompt).await?;

    println!("Response: {}", response);

    Ok(())
}
