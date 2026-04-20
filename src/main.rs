mod config;
mod llm;
mod tui;
mod types;

use anyhow::Result;

use crate::{
    config::LLMConfig,
    llm::tools::file_explorer::{
        find_files::AgentToolFindFiles, get_file_info::AgentToolGetFileInfo,
        list_directory::AgentToolListDirectory,
    },
    llm::tools::file_reader::AgentToolFileReader,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = LLMConfig::from_file("./src/config/config.toml")?;
    let project_path = std::env::current_dir()?;

    let tools: Vec<Box<dyn rig::tool::ToolDyn>> = vec![
        Box::new(AgentToolListDirectory::new(project_path.clone())),
        Box::new(AgentToolFindFiles::new(project_path.clone())),
        Box::new(AgentToolGetFileInfo::new(project_path.clone())),
        Box::new(AgentToolFileReader::new(project_path)),
    ];

    tui::run(config, tools).await
}
