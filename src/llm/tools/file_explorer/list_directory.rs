use std::collections::HashMap;

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::llm::tools::file_explorer::{
    DEFAULT_MAX_FILES, FileExplorerCore, FileExplorerResult, FileExplorerToolError,
};

#[derive(Debug, Clone)]
pub struct AgentToolListDirectory {
    core: FileExplorerCore,
}

#[derive(Debug, Deserialize)]
pub struct ListDirectoryArgs {
    pub path: Option<String>,
    pub recursive: Option<bool>,
    pub max_files: Option<usize>,
}

impl AgentToolListDirectory {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self {
            core: FileExplorerCore::new(project_path),
        }
    }

    async fn execute(&self, args: &ListDirectoryArgs) -> Result<FileExplorerResult> {
        let target_path = self.core.resolve_path(args.path.as_deref());

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("Path does not exist: {}", target_path.display())],
                ..Default::default()
            });
        }

        let recursive = args.recursive.unwrap_or(false);
        let max_files = args.max_files.unwrap_or(DEFAULT_MAX_FILES);
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut file_types = HashMap::new();

        if recursive {
            for entry in WalkDir::new(&target_path).max_depth(10) {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if entry.file_type().is_file() {
                    let file_info = self.core.create_file_info(path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type().is_dir() && path != target_path {
                    let relative_path = path
                        .strip_prefix(&self.core.project_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        } else {
            for entry in std::fs::read_dir(&target_path)? {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if entry.file_type()?.is_file() {
                    let file_info = self.core.create_file_info(&path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type()?.is_dir() {
                    let relative_path = path
                        .strip_prefix(&self.core.project_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        }

        let insights = self
            .core
            .generate_insights(&files, &directories, &file_types);

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories,
            insights,
            file_types,
        })
    }
}

impl Tool for AgentToolListDirectory {
    const NAME: &'static str = "list_directory";

    type Error = FileExplorerToolError;
    type Args = ListDirectoryArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "List directory contents with optional recursive traversal and file type insights."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Target path (relative to project root)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to recursively search subdirectories (default false)"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum number of files to return (default 100)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.execute(&args)
            .await
            .map_err(|_e| FileExplorerToolError)
    }
}
