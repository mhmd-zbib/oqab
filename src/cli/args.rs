use clap::{Parser, ValueEnum};
use anyhow::{Context, Result};
use thiserror::Error;
use log::{info, warn, debug};
use std::path::Path;
use crate::core::traversal::TraversalMode;
use crate::core::config::FileSearchConfig;
use regex;

/// Errors related to command-line argument processing
#[derive(Error, Debug)]
pub enum ArgsError {
    #[error("Failed to parse command line arguments: {0}")]
    ParseError(String),
    
    #[error("Failed to load configuration: {0}")]
    ConfigLoadError(String),
    
    #[error("Failed to save configuration: {0}")]
    ConfigSaveError(String),
    
    #[error("Invalid argument value: {0}")]
    InvalidValue(String),
}

/// Command line arguments for the Oqab application
#[derive(Parser, Debug)]
#[command(name = "Oqab")]
#[command(author = "Dev Team")]
#[command(version = "1.0.0")]
#[command(about = "High-performance file search utility")]
#[command(disable_help_flag = true)]
#[command(override_usage = "oqab [OPTIONS] [QUERY]\n       oqab [QUERY]\n       oqab --grep PATTERN [OPTIONS]")]
pub struct Args {
    /// Search query (file name pattern or text to search for)
    #[arg(index = 1)]
    pub query: Option<String>,
    /// Display help information
    #[arg(short = 'h', long = "help")]
    pub help: bool,

    /// Directory to search (defaults to current directory, use '/' for root)
    #[arg(short = 'p', long = "path")]
    pub path: Option<String>,

    /// File extension to search for
    #[arg(short = 'e', long = "ext")]
    pub extension: Option<String>,

    /// File name pattern to search for
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
    
    /// Text pattern to search for within files (grep-like functionality)
    #[arg(short = 'g', long = "grep")]
    pub pattern: Option<String>,
    
    /// Case insensitive search
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,
    
    /// Show line numbers in search results
    #[arg(long = "line-number")]
    pub line_number: bool,
    
    /// Show only filenames of files containing the pattern
    #[arg(long = "files-with-matches")]
    pub files_with_matches: bool,
    
    /// Use advanced search algorithm
    #[arg(short = 'a', long = "advanced")]
    pub advanced: bool,
    
    /// Suppress progress output
    #[arg(short = 's', long = "silent")]
    pub silent: bool,
    
    /// Number of worker threads
    #[arg(short = 'w', long = "workers")]
    pub workers: Option<usize>,
    
    /// Load configuration from file
    #[arg(short = 'c', long = "config")]
    pub config_file: Option<String>,
    
    /// Save current configuration to file
    #[arg(long = "save-config")]
    pub save_config_file: Option<String>,

    /// Traversal type
    #[arg(short = 't', long = "traversal")]
    pub traversal: Option<TraversalType>,

    /// Quiet mode
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// No recursive search
    #[arg(short = 'r', long = "no-recursive")]
    pub no_recursive: bool,

    /// Follow symlinks
    #[arg(short = 'f', long = "follow-symlinks")]
    pub follow_symlinks: bool,
    
    /// Filter by minimum file size (e.g., "10kb", "5mb")
    #[arg(long = "min-size")]
    pub min_size: Option<String>,
    
    /// Filter by maximum file size (e.g., "10kb", "5mb")
    #[arg(long = "max-size")]
    pub max_size: Option<String>,
    
    /// Filter by modified after date (YYYY-MM-DD)
    #[arg(long = "newer-than")]
    pub newer_than: Option<String>,
    
    /// Filter by modified before date (YYYY-MM-DD)
    #[arg(long = "older-than")]
    pub older_than: Option<String>,
}

/// Available traversal strategies for directory searching
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TraversalType {
    /// Traverse directories level by level
    #[value(name = "bfs")]
    BreadthFirst,
    /// Process each directory branch fully before moving to siblings
    #[value(name = "dfs")]
    DepthFirst,
}

impl From<TraversalType> for TraversalMode {
    fn from(value: TraversalType) -> Self {
        match value {
            TraversalType::BreadthFirst => TraversalMode::BreadthFirst,
            TraversalType::DepthFirst => TraversalMode::DepthFirst,
        }
    }
}

impl Args {
    /// Parse command line arguments
    pub fn parse() -> Result<Self> {
        Self::try_parse()
            .map_err(|e| ArgsError::ParseError(e.to_string()).into())
    }
    
    /// Convert CLI arguments to a search configuration
    pub fn to_config(&self) -> FileSearchConfig {
        let mut config = FileSearchConfig::new();
        
        // Apply CLI args to the config
        self.apply_to_config(&mut config);
        
        config
    }
    
    /// Apply CLI arguments to an existing configuration
    fn apply_to_config(&self, config: &mut FileSearchConfig) {
        // Basic settings
        if let Some(path) = &self.path {
            config.path = Some(path.clone());
        }
        config.file_extension = self.extension.clone();
        config.file_name = self.name.clone();
        config.pattern = self.pattern.clone();
        config.ignore_case = self.ignore_case;
        config.line_number = self.line_number;
        config.files_with_matches = self.files_with_matches;
        config.help = self.help;
        
        // Performance settings
        if let Some(threads) = self.workers {
            config.thread_count = Some(threads);
        }
        
        // Advanced settings
        config.advanced_search = self.advanced;
        if let Some(traversal_type) = self.traversal {
            config.traversal_mode = traversal_type.into();
        }
        
        // Size filters
        if let Some(min_size) = &self.min_size {
            if let Ok(size) = Self::parse_size(min_size) {
                config.min_size = Some(size);
            }
        }
        
        if let Some(max_size) = &self.max_size {
            if let Ok(size) = Self::parse_size(max_size) {
                config.max_size = Some(size);
            }
        }
        
        // Date filters
        config.newer_than = self.newer_than.clone();
        config.older_than = self.older_than.clone();
        
        // Other settings
        config.show_progress = !self.quiet && !self.silent;
        config.recursive = !self.no_recursive;
        config.follow_symlinks = self.follow_symlinks;
    }
    
    /// Parse a human-readable size string into bytes
    fn parse_size(size_str: &str) -> Result<u64> {
        let size_str = size_str.trim().to_lowercase();
        
        // Regular expression to match a number followed by an optional unit
        let re = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*([kmgt]?b?)?$").unwrap();
        
        if let Some(caps) = re.captures(&size_str) {
            let value: f64 = caps.get(1)
                .map_or("0", |m| m.as_str())
                .parse()
                .unwrap_or(0.0);
                
            let unit = caps.get(2).map_or("", |m| m.as_str());
            
            let multiplier: u64 = match unit {
                "k" | "kb" => 1_024,
                "m" | "mb" => 1_024 * 1_024,
                "g" | "gb" => 1_024 * 1_024 * 1_024,
                "t" | "tb" => 1_099_511_627_776, // 1024^4
                _ => 1, // Default to bytes
            };
            
            Ok((value * multiplier as f64) as u64)
        } else {
            Err(ArgsError::InvalidValue(format!("Invalid size format: {}", size_str)).into())
        }
    }
    
    /// Process command-line arguments, loading from config file if specified
    pub fn process(&self) -> Result<FileSearchConfig> {
        // Validate required arguments
        self.validate()?;
        
        // First convert CLI args to a config
        let mut config = self.to_config();
        
        // Handle positional argument if present
        if let Some(query) = &self.query {
            // If no explicit search type is specified, use the query as file name pattern
            if config.file_name.is_none() && config.file_extension.is_none() && config.pattern.is_none() {
                config.file_name = Some(query.clone());
            }
        }
        
        // If a config file is specified, load and merge it
        if let Some(config_file) = &self.config_file {
            // Check if the config file exists
            let path = Path::new(config_file);
            if !path.exists() {
                return Err(ArgsError::ConfigLoadError(
                    format!("Config file not found: {}", config_file)
                ).into());
            }
            
            let loaded_config = FileSearchConfig::load_from_file(config_file)
                .with_context(|| format!("Failed to load configuration file: {}", config_file))?;
            
            // Merge the loaded config with CLI args (CLI args take precedence)
            config = self.merge_with_config(loaded_config);
            debug!("Merged configuration from file and command line arguments");
        }
        
        // Process the save config request if present
        if let Some(save_path) = &self.save_config_file {
            debug!("Will save configuration to: {}", save_path);
        }
        
        // Final validation
        self.validate_config(&config)?;
        
        Ok(config)
    }
    
    /// Validate command-line arguments
    fn validate(&self) -> Result<()> {
        // Validate worker threads
        if let Some(workers) = self.workers {
            if workers == 0 {
                return Err(ArgsError::InvalidValue(
                    "Worker thread count must be greater than 0".to_string()
                ).into());
            }
        }
        
        // Validate that path exists if specified
        if let Some(path) = &self.path {
            let p = Path::new(path);
            if !p.exists() {
                return Err(ArgsError::InvalidValue(
                    format!("Specified path does not exist: {}", path)
                ).into());
            }
        }
        
        Ok(())
    }
    
    /// Validate the generated configuration
    fn validate_config(&self, config: &FileSearchConfig) -> Result<()> {
        // Check if search criteria is present
        if config.file_extension.is_none() && config.file_name.is_none() && config.pattern.is_none() && !self.help {
            warn!("No search criteria specified, behavior may be undefined");
        }
        
        Ok(())
    }
    
    /// Merge CLI arguments with a loaded configuration
    fn merge_with_config(&self, loaded: FileSearchConfig) -> FileSearchConfig {
        let mut merged = loaded.clone();
        
        // Apply CLI args to the loaded config
        // CLI args take precedence over loaded config values
        self.selective_apply_to_config(&mut merged);
        
        merged
    }
    
    /// Selectively apply CLI arguments to an existing configuration
    /// Only overrides values that were explicitly set on command line
    fn selective_apply_to_config(&self, config: &mut FileSearchConfig) {
        // Path - only override if specified in CLI
        if let Some(path) = &self.path {
            config.path = Some(path.clone());
        }
        
        // File extension - only override if specified in CLI
        if self.extension.is_some() {
            config.file_extension = self.extension.clone();
        }
        
        // File name - only override if specified in CLI
        if self.name.is_some() {
            config.file_name = self.name.clone();
        }
        
        // Pattern - only override if specified in CLI
        if self.pattern.is_some() {
            config.pattern = self.pattern.clone();
        }
        
        // Search options - override if flags are set
        if self.ignore_case {
            config.ignore_case = true;
        }
        
        if self.line_number {
            config.line_number = true;
        }
        
        if self.files_with_matches {
            config.files_with_matches = true;
        }
        
        // Thread count - only override if specified in CLI
        if let Some(threads) = self.workers {
            config.thread_count = Some(threads);
        }
        
        // Traversal strategy - only override if specified in CLI
        if let Some(traversal_type) = self.traversal {
            config.traversal_mode = traversal_type.into();
        }
        
        // Boolean settings require special handling
        
        // Help flag - override if help flag is set
        if self.help {
            config.help = true;
        }
        
        // Advanced search - override if advanced flag is set
        if self.advanced {
            config.advanced_search = true;
        }
        
        // Progress - override if quiet flag is set
        if self.quiet {
            config.show_progress = false;
        }
        
        // Recursive - override if no-recursive flag is set
        if self.no_recursive {
            config.recursive = false;
        }
        
        // Follow symlinks - override if follow-symlinks flag is set
        if self.follow_symlinks {
            config.follow_symlinks = true;
        }
    }
    
    /// Save current configuration to a file
    pub fn save_config(&self, config: &FileSearchConfig) -> Result<()> {
        if let Some(save_path) = &self.save_config_file {
            let path = Path::new(save_path);
            
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| ArgsError::ConfigSaveError(
                            format!("Failed to create directory: {}", parent.display())
                        ))?;
                }
            }
            
            // Save the configuration
            config.save_to_file(save_path)
                .with_context(|| ArgsError::ConfigSaveError(
                    format!("Failed to save configuration to: {}", save_path)
                ))?;
                
            info!("Configuration saved to: {}", save_path);
        }
        
        Ok(())
    }
} 