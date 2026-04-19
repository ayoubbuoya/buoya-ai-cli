mod config;
mod llm;
mod types;

use anyhow::Result;

use crate::{config::LLMConfig, llm::tools::file_explorer::{find_files::AgentToolFindFiles, get_file_info::AgentToolGetFileInfo, list_directory::AgentToolListDirectory}};


#[tokio::main]
async fn main() -> Result<()> {
    let config = LLMConfig::from_file("./src/config/config.toml")?;

    println!("Config: {:?}", config);

    let project_path = std::env::current_dir()?;

    println!("Project path: {}", project_path.display());

    let agent = llm::agent::LLMAgent::new(
        config,
        vec![
            Box::new(AgentToolListDirectory::new(project_path.clone())),
            Box::new(AgentToolFindFiles::new(project_path.clone())),
            Box::new(AgentToolGetFileInfo::new(project_path)),
        ],
    )?;

    let prompt = "Explore the current project directory structure. List the top-level files and directories, then tell me what kind of project this is based on the files you see.";

    let response = agent.prompt(prompt).await?;

    println!("Response: {}", response);

    Ok(())
}
