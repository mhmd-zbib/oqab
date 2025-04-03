use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use crate::search::TraversalStrategy;

/// Configuration for file search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchConfig {
    /// Path to search in (defaults to current directory)
    pub path: Option<String>,
    
    /// File extension to filter by
    pub file_extension: Option<String>,
    
    /// File name pattern to filter by
    pub file_name: Option<String>,
    
    /// Whether to use advanced search features
    pub advanced_search: bool,
    
    /// Number of threads to use for parallel search
    pub thread_count: Option<usize>,
    
    /// Whether to show progress during search
    pub show_progress: bool,
    
    /// Whether to search recursively in subdirectories
    pub recursive: bool,
    
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    
    /// Advanced options
    pub traversal_strategy: Option<TraversalStrategy>,
}

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
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
            
        let config: Self = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path))?;
            
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let serialized = serde_json::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
            
        fs::write(path, serialized)
            .with_context(|| format!("Failed to write config to file: {}", path))?;
            
        Ok(())
    }
    
    /// Get the search path or a default
    pub fn get_path(&self) -> String {
        self.path.clone().unwrap_or_else(|| ".".to_string())
    }
}

impl Default for FileSearchConfig {
    fn default() -> Self {
        Self::new()
    }
} 