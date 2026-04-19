pub mod find_files;
pub mod get_file_info;
pub mod list_directory;

use std::{collections::HashMap, path::Path};

use anyhow::Result;
use serde::Serialize;
use thiserror::Error;

use crate::types::FileInfo;

pub const DEFAULT_MAX_FILES: usize = 100;

#[derive(Debug, Clone)]
pub struct FileExplorerCore {
    pub project_path: std::path::PathBuf,
}

impl FileExplorerCore {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }

    pub fn resolve_path(&self, relative: Option<&str>) -> std::path::PathBuf {
        relative
            .map(|path| self.project_path.join(path))
            .unwrap_or_else(|| self.project_path.clone())
    }

    pub fn create_file_info(&self, path: &Path) -> Result<FileInfo> {
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

    pub fn generate_insights(
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
            for (ext, count) in file_types {
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

    pub fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }

        file_name.to_lowercase().contains(&pattern.to_lowercase())
    }
}

#[derive(Debug, Error)]
#[error("file explorer tool error")]
pub struct FileExplorerToolError;

#[derive(Debug, Serialize, Default)]
pub struct FileExplorerResult {
    pub files: Vec<FileInfo>,
    pub directories: Vec<String>,
    pub total_count: usize,
    pub insights: Vec<String>,
    pub file_types: HashMap<String, usize>,
}
