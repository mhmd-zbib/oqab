use clap::{Parser, ValueEnum};
use anyhow::{Context, Result};
use thiserror::Error;
use log::{info, warn, debug};
use std::path::Path;
use crate::search::TraversalStrategy;
use crate::config::FileSearchConfig;

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
#[command(about = "High-performance file finding utility")]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Display help information
    #[arg(short = 'h', long = "help")]
    pub help: bool,

    /// Directory to search
    #[arg(short = 'p', long = "path")]
    pub path: Option<String>,

    /// File extension to search for
    #[arg(short = 'e', long = "ext")]
    pub extension: Option<String>,

    /// File name pattern to search for
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
    
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

impl From<TraversalType> for TraversalStrategy {
    fn from(value: TraversalType) -> Self {
        match value {
            TraversalType::BreadthFirst => TraversalStrategy::BreadthFirst,
            TraversalType::DepthFirst => TraversalStrategy::DepthFirst,
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
        
        // Performance settings
        if let Some(threads) = self.workers {
            config.thread_count = Some(threads);
        }
        
        // Advanced settings
        if let Some(traversal_type) = self.traversal {
            config.traversal_strategy = Some(traversal_type.into());
        }
        
        // Other settings
        config.show_progress = !self.quiet;
        config.recursive = !self.no_recursive;
        config.follow_symlinks = self.follow_symlinks;
    }
    
    /// Process command-line arguments, loading from config file if specified
    pub fn process(&self) -> Result<FileSearchConfig> {
        // Validate required arguments
        self.validate()?;
        
        // First convert CLI args to a config
        let mut config = self.to_config();
        
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
        if config.file_extension.is_none() && config.file_name.is_none() && !self.help {
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
        
        // Thread count - only override if specified in CLI
        if let Some(threads) = self.workers {
            config.thread_count = Some(threads);
        }
        
        // Traversal strategy - only override if specified in CLI
        if let Some(traversal_type) = self.traversal {
            config.traversal_strategy = Some(traversal_type.into());
        }
        
        // Boolean settings require special handling
        
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