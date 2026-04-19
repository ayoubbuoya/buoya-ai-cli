use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;

use crate::llm::tools::file_explorer::{
    FileExplorerCore, FileExplorerResult, FileExplorerToolError,
};

#[derive(Debug, Clone)]
pub struct AgentToolGetFileInfo {
    core: FileExplorerCore,
}

#[derive(Debug, Deserialize)]
pub struct GetFileInfoArgs {
    pub path: String,
}

impl AgentToolGetFileInfo {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self {
            core: FileExplorerCore::new(project_path),
        }
    }

    async fn execute(&self, args: &GetFileInfoArgs) -> Result<FileExplorerResult> {
        let target_path = self.core.resolve_path(Some(&args.path));

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("File does not exist: {}", target_path.display())],
                ..Default::default()
            });
        }

        if !target_path.is_file() {
            return Ok(FileExplorerResult {
                insights: vec![format!("Path is not a file: {}", target_path.display())],
                ..Default::default()
            });
        }

        let file_info = self.core.create_file_info(&target_path)?;
        let mut file_types = std::collections::HashMap::new();
        if let Some(ext) = &file_info.extension {
            file_types.insert(ext.clone(), 1);
        }

        let insights = vec![
            format!("File path: {}", file_info.path.display()),
            format!("File size: {} bytes", file_info.size),
            format!(
                "File extension: {}",
                file_info.extension.as_deref().unwrap_or("none")
            ),
            format!(
                "Last modified: {}",
                file_info.last_modified.as_deref().unwrap_or("unknown")
            ),
        ];

        Ok(FileExplorerResult {
            total_count: 1,
            files: vec![file_info],
            directories: Vec::new(),
            insights,
            file_types,
        })
    }
}

impl Tool for AgentToolGetFileInfo {
    const NAME: &'static str = "get_file_info";

    type Error = FileExplorerToolError;
    type Args = GetFileInfoArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get metadata for a single file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path (relative to project root)"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.execute(&args)
            .await
            .map_err(|_e| FileExplorerToolError)
    }
}
