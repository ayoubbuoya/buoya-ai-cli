use std::{collections::HashMap, path::Path};

use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::types::FileInfo;

pub const DEFAULT_MAX_FILES: usize = 100;

/// File exploration tool
#[derive(Debug, Clone)]
pub struct AgentToolFileExplorer {
    project_path: std::path::PathBuf,
}

#[derive(Debug, Error)]
#[error("file explorer tool error")]
pub struct FileExplorerToolError;

/// File exploration parameters
#[derive(Debug, Deserialize)]
pub struct FileExplorerArgs {
    pub action: String, // "list_directory", "find_files", "get_file_info"
    pub path: Option<String>,
    pub pattern: Option<String>,
    pub recursive: Option<bool>,
    pub max_files: Option<usize>,
}

/// File exploration result
#[derive(Debug, Serialize, Default)]
pub struct FileExplorerResult {
    pub files: Vec<FileInfo>,
    pub directories: Vec<String>,
    pub total_count: usize,
    pub insights: Vec<String>,
    pub file_types: HashMap<String, usize>,
}

impl AgentToolFileExplorer {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }

    /// List directory contents with optional recursion and file type insights
    async fn list_directory(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let target_path = if let Some(path) = &args.path {
            self.project_path.join(path)
        } else {
            self.project_path.clone()
        };

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
            // Recursive traversal, limit depth to 10
            for entry in WalkDir::new(&target_path).max_depth(10) {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if entry.file_type().is_file() {
                    let file_info = self.create_file_info(path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type().is_dir() && path != target_path {
                    let relative_path = path
                        .strip_prefix(&self.project_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        } else {
            // Non-recursive, only list current directory
            for entry in std::fs::read_dir(&target_path)? {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if entry.file_type()?.is_file() {
                    let file_info = self.create_file_info(&path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type()?.is_dir() {
                    let relative_path = path
                        .strip_prefix(&self.project_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        }

        let insights = self.generate_insights(&files, &directories, &file_types);

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories,
            insights,
            file_types,
        })
    }

    async fn find_files(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let pattern = args
            .pattern
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("find_files action requires pattern parameter"))?;

        let search_path = if let Some(path) = &args.path {
            self.project_path.join(path)
        } else {
            self.project_path.clone()
        };

        if !search_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!(
                    "Search path does not exist: {}",
                    search_path.display()
                )],
                ..Default::default()
            });
        }

        let max_files = args.max_files.unwrap_or(100);
        let mut files = Vec::new();
        let mut file_types = HashMap::new();

        // Use walkdir for recursive search, limit depth to 5
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

            // Simple pattern matching
            if self.matches_pattern(file_name, pattern) {
                let file_info = self.create_file_info(path)?;
                if let Some(ext) = &file_info.extension {
                    *file_types.entry(ext.clone()).or_insert(0) += 1;
                }
                files.push(file_info);
            }
        }

        let insights = vec![
            format!("Search pattern: {}", pattern),
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

    async fn get_file_info(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let file_path = args
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("get_file_info action requires path parameter"))?;

        let target_path = self.project_path.join(file_path);

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

        let file_info = self.create_file_info(&target_path)?;
        let mut file_types = HashMap::new();
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

    /// Create FileInfo from a file path
    fn create_file_info(&self, path: &Path) -> Result<FileInfo> {
        let metadata = std::fs::metadata(path)?;

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let relative_path = path
            .strip_prefix(&self.project_path)
            .unwrap_or(path)
            .to_path_buf();

        let last_modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string());

        Ok(FileInfo {
            path: relative_path,
            name,
            size: metadata.len(),
            extension,
            last_modified,
        })
    }

    /// Generate insights based on file and directory information
    fn generate_insights(
        &self,
        files: &[FileInfo],
        directories: &[String],
        file_types: &HashMap<String, usize>,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        insights.push(format!(
            "Found {} files and {} directories",
            files.len(),
            directories.len()
        ));

        if !file_types.is_empty() {
            let mut type_summary = String::new();
            for (ext, count) in file_types.iter() {
                if !type_summary.is_empty() {
                    type_summary.push_str(", ");
                }
                type_summary.push_str(&format!("{}: {}", ext, count));
            }
            insights.push(format!("File type distribution: {}", type_summary));
        }

        let total_size: u64 = files.iter().map(|f| f.size).sum();
        if total_size > 0 {
            insights.push(format!("Total file size: {} bytes", total_size));
        }

        insights
    }

    /// Match file name against a pattern (supports simple wildcard *)
    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }

        // Contains matching
        file_name.to_lowercase().contains(&pattern.to_lowercase())
    }
}

impl Tool for AgentToolFileExplorer {
    const NAME: &'static str = "file_explorer";

    type Error = FileExplorerToolError;
    type Args = FileExplorerArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description:"Explore project file structure, list directory contents, find specific file patterns. Supports recursive search and file filtering."
                    .to_string(),
               parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_directory", "find_files", "get_file_info"],
                        "description": "Action type to execute: list_directory (list directory), find_files (find files), get_file_info (get file info)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Target path (relative to project root)"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "File search pattern (for find_files operation)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to recursively search subdirectories (default false)"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum number of files to return (default 100)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("File explorer tool called with args: {:?}", args);

        match args.action.as_str() {
            "list_directory" => self
                .list_directory(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "find_files" => self
                .find_files(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "get_file_info" => self
                .get_file_info(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            _ => Err(FileExplorerToolError),
        }
    }
}
