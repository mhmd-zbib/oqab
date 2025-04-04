use std::fs;
use std::path::Path;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use crate::search::TraversalStrategy;

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
    
    /// Advanced options
    #[serde(default)]
    pub traversal_strategy: Option<TraversalStrategy>,
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
            traversal_strategy: None,
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