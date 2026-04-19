use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub extension: Option<String>,
    pub last_modified: Option<String>,
}

/// Directory information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub file_count: usize,
    pub subdirectory_count: usize,
    pub total_size: u64,
}
