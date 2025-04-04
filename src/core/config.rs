use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::core::traversal::TraversalMode;

/// Errors that can occur during configuration operations
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(String),
    
    #[error("Failed to parse config file: {0}")]
    ParseError(String),
    
    #[error("Failed to write config file: {0}")]
    WriteError(String),
}

/// Configuration for file search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchConfig {
    /// Path to search in (defaults to current directory)
    #[serde(default)]
    pub path: Option<String>,
    
    /// File extension to filter by
    #[serde(default)]
    pub file_extension: Option<String>,
    
    /// File name pattern to filter by
    #[serde(default)]
    pub file_name: Option<String>,
    
    /// Whether to use advanced search features
    #[serde(default)]
    pub advanced_search: bool,
    
    /// Number of threads to use for parallel search
    #[serde(default)]
    pub thread_count: Option<usize>,
    
    /// Whether to show progress during search
    #[serde(default = "default_show_progress")]
    pub show_progress: bool,
    
    /// Whether to search recursively in subdirectories
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    
    /// Whether to follow symbolic links
    #[serde(default)]
    pub follow_symlinks: bool,
    
    /// Traversal strategy to use
    #[serde(default)]
    pub traversal_mode: TraversalMode,
    
    /// Minimum file size in bytes
    #[serde(default)]
    pub min_size: Option<u64>,
    
    /// Maximum file size in bytes
    #[serde(default)]
    pub max_size: Option<u64>,
    
    /// Modified after this date (ISO format: YYYY-MM-DD)
    #[serde(default)]
    pub newer_than: Option<String>,
    
    /// Modified before this date (ISO format: YYYY-MM-DD)
    #[serde(default)]
    pub older_than: Option<String>,
}

// Helper functions for serde defaults
fn default_show_progress() -> bool { true }
fn default_recursive() -> bool { true }

impl FileSearchConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            path: None,
            file_extension: None,
            file_name: None,
            advanced_search: false,
            thread_count: None,
            show_progress: true,
            recursive: true,
            follow_symlinks: false,
            traversal_mode: TraversalMode::default(),
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
        }
    }
    
    /// Load configuration from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_display = path.as_ref().display().to_string();
        
        let contents = fs::read_to_string(&path)
            .with_context(|| ConfigError::ReadError(path_display.clone()))?;
            
        let config: Self = serde_json::from_str(&contents)
            .with_context(|| ConfigError::ParseError(path_display))?;
            
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_display = path.as_ref().display().to_string();
        
        let serialized = serde_json::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
            
        fs::write(&path, serialized)
            .with_context(|| ConfigError::WriteError(path_display))?;
            
        Ok(())
    }
    
    /// Get the search path or the default "." path
    pub fn get_path(&self) -> &str {
        self.path.as_deref().unwrap_or(".")
    }
}

impl Default for FileSearchConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Application runtime configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Root directory to search
    pub root_dir: PathBuf,
    
    /// File extension to filter by
    pub extension: Option<String>,
    
    /// File name to filter by
    pub name: Option<String>,
    
    /// Regular expression pattern to filter by
    pub pattern: Option<String>,
    
    /// Minimum file size in bytes
    pub min_size: Option<u64>,
    
    /// Maximum file size in bytes
    pub max_size: Option<u64>,
    
    /// Modified after this date (ISO format: YYYY-MM-DD)
    pub newer_than: Option<String>,
    
    /// Modified before this date (ISO format: YYYY-MM-DD)
    pub older_than: Option<String>,
    
    /// Size to filter by (legacy)
    pub size: Option<u64>,
    
    /// Maximum depth to search
    pub depth: Option<usize>,
    
    /// Number of threads to use
    pub threads: Option<usize>,
    
    /// Whether to follow symbolic links
    pub follow_links: Option<bool>,
    
    /// Whether to show progress during search
    pub show_progress: Option<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            root_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            extension: None,
            name: None,
            pattern: None,
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            size: None,
            depth: None,
            threads: Some(num_cpus::get()),
            follow_links: Some(false),
            show_progress: Some(true),
        }
    }
} 