use std::collections::HashMap;

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::llm::tools::file_explorer::{
    DEFAULT_MAX_FILES, FileExplorerCore, FileExplorerResult, FileExplorerToolError,
};

#[derive(Debug, Clone)]
pub struct AgentToolFindFiles {
    core: FileExplorerCore,
}

#[derive(Debug, Deserialize)]
pub struct FindFilesArgs {
    pub pattern: String,
    pub path: Option<String>,
    pub max_files: Option<usize>,
}

impl AgentToolFindFiles {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self {
            core: FileExplorerCore::new(project_path),
        }
    }

    async fn execute(&self, args: &FindFilesArgs) -> Result<FileExplorerResult> {
        let search_path = self.core.resolve_path(args.path.as_deref());

        if !search_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!(
                    "Search path does not exist: {}",
                    search_path.display()
                )],
                ..Default::default()
            });
        }

        let max_files = args.max_files.unwrap_or(DEFAULT_MAX_FILES);
        let mut files = Vec::new();
        let mut file_types = HashMap::new();

        for entry in WalkDir::new(&search_path).max_depth(5) {
            if files.len() >= max_files {
                break;
            }

            let entry = entry?;
            let path = entry.path();

            if !entry.file_type().is_file() {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if self.core.matches_pattern(file_name, &args.pattern) {
                let file_info = self.core.create_file_info(path)?;
                if let Some(ext) = &file_info.extension {
                    *file_types.entry(ext.clone()).or_insert(0) += 1;
                }
                files.push(file_info);
            }
        }

        let insights = vec![
            format!("Search pattern: {}", args.pattern),
            format!("Search path: {}", search_path.display()),
            format!("Found {} matching files", files.len()),
        ];

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories: Vec::new(),
            insights,
            file_types,
        })
    }
}

impl Tool for AgentToolFindFiles {
    const NAME: &'static str = "find_files";

    type Error = FileExplorerToolError;
    type Args = FindFilesArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Find files by name pattern (supports wildcard *) within a directory tree."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "File search pattern"
                    },
                    "path": {
                        "type": "string",
                        "description": "Search path (relative to project root)"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum number of files to return (default 100)"
                    }
                },
                "required": ["pattern"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.execute(&args)
            .await
            .map_err(|_e| FileExplorerToolError)
    }
}
