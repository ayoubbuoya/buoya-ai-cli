use anyhow::{Context, Result};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AgentToolFileReader {
    project_path: std::path::PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct FileReaderArgs {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FileReaderOutput {
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub returned_lines: usize,
    pub total_lines: usize,
    pub has_more: bool,
    pub content: String,
    pub insights: Vec<String>,
}

#[derive(Debug, Error)]
#[error("file reader tool error")]
pub struct FileReaderToolError;

impl AgentToolFileReader {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }

    fn resolve_safe_path(&self, relative_path: &str) -> Result<std::path::PathBuf> {
        let joined = self.project_path.join(relative_path);
        let resolved = joined
            .canonicalize()
            .with_context(|| format!("failed to resolve file path: {}", joined.display()))?;
        let project_root = self
            .project_path
            .canonicalize()
            .with_context(|| "failed to resolve project root".to_string())?;

        if !resolved.starts_with(&project_root) {
            anyhow::bail!("path is outside the project root");
        }

        Ok(resolved)
    }

    async fn execute(&self, args: &FileReaderArgs) -> Result<FileReaderOutput> {
        let file_path = self.resolve_safe_path(&args.path)?;

        if !file_path.exists() {
            anyhow::bail!("file does not exist: {}", file_path.display());
        }

        if !file_path.is_file() {
            anyhow::bail!("path is not a file: {}", file_path.display());
        }

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("failed to read file: {}", file_path.display()))?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let offset = args.offset.unwrap_or(0).min(total_lines);
        let limit = args.limit.unwrap_or(total_lines.saturating_sub(offset));
        let end = offset.saturating_add(limit).min(total_lines);

        let selected = lines[offset..end].join("\n");
        let relative_path = file_path
            .strip_prefix(&self.project_path)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let has_more = end < total_lines;
        let insights = vec![
            format!("Read file: {}", relative_path),
            format!("Lines returned: {} to {}", offset + 1, end),
            format!("Total lines: {}", total_lines),
            if has_more {
                format!("More lines available starting from line {}", end + 1)
            } else {
                "Returned the requested range with no remaining lines".to_string()
            },
        ];

        Ok(FileReaderOutput {
            path: relative_path,
            start_line: offset + 1,
            end_line: end,
            returned_lines: end.saturating_sub(offset),
            total_lines,
            has_more,
            content: selected,
            insights,
        })
    }
}

impl Tool for AgentToolFileReader {
    const NAME: &'static str = "file_reader";

    type Error = FileReaderToolError;
    type Args = FileReaderArgs;
    type Output = FileReaderOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read a file fully or by line range using offset and limit.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to project root"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Zero-based line offset to start reading from (default 0)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of lines to return. If omitted, returns all remaining lines up to a safe default."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.execute(&args).await.map_err(|_e| FileReaderToolError)
    }
}
