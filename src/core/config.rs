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
    
    /// Text pattern to search for within files (grep-like functionality)
    #[serde(default)]
    pub pattern: Option<String>,
    
    /// Whether to use case-insensitive search
    #[serde(default)]
    pub ignore_case: bool,
    
    /// Whether to show line numbers in search results
    #[serde(default)]
    pub line_number: bool,
    
    /// Whether to show only filenames of files containing the pattern
    #[serde(default)]
    pub files_with_matches: bool,

    /// Whether to use fuzzy matching for file names
    #[serde(default)]
    pub fuzzy: bool,

    /// Fuzzy match threshold (0-100, higher means stricter matching)
    #[serde(default)]
    pub fuzzy_threshold: Option<u8>,
    
    /// Whether to display help information
    #[serde(default)]
    pub help: bool,
    
    /// Whether to use advanced search features
    #[serde(default)]
    pub advanced_search: bool,
    
    /// Number of threads to use for parallel search
    #[serde(default)]
    pub thread_count: Option<usize>,
    
    /// Whether to show progress during search
    #[serde(default = "default_show_progress")]
    pub show_progress: bool,
    
    /// Whether to use quiet mode (less verbose output)
    #[serde(default)]
    pub quiet_mode: bool,
    
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
            pattern: None,
            ignore_case: false,
            line_number: false,
            files_with_matches: false,
            help: false,
            advanced_search: false,
            thread_count: None,
            show_progress: true,
            quiet_mode: false,
            recursive: true,
            follow_symlinks: false,
            traversal_mode: TraversalMode::default(),
            min_size: None,
            max_size: None,
            newer_than: None,
            older_than: None,
            fuzzy: false,
            fuzzy_threshold: None,
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
    
    /// Whether to use quiet mode (less verbose output)
    pub quiet: Option<bool>,
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
            quiet: Some(false),
        }
    }
} 