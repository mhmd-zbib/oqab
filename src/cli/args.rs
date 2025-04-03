use clap::{Parser, ValueEnum};
use anyhow::{Context, Result};
use log::info;
use crate::search::TraversalStrategy;
use crate::config::FileSearchConfig;

/// Command line arguments for the HyperSearch application
#[derive(Parser, Debug)]
#[command(name = "HyperSearch")]
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
        Ok(Self::try_parse()?)
    }
    
    /// Convert CLI arguments to a search configuration
    pub fn to_config(&self) -> FileSearchConfig {
        let mut config = FileSearchConfig::new();
        
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
            config.traversal_strategy = Some(match traversal_type {
                TraversalType::BreadthFirst => TraversalStrategy::BreadthFirst,
                TraversalType::DepthFirst => TraversalStrategy::DepthFirst,
            });
        }
        
        // Other settings
        config.show_progress = !self.quiet;
        config.recursive = !self.no_recursive;
        config.follow_symlinks = self.follow_symlinks;
        
        config
    }
    
    /// Process the config file if specified
    pub fn process_config_file(&self) -> Result<Option<FileSearchConfig>> {
        // Check if config file is specified
        if let Some(config_path) = &self.config_file {
            let path_str = config_path.clone();
            
            if let Some(_save_path) = &self.save_config_file {
                // We'll save the config later, but return None for now
                return Ok(None);
            }
            
            // Load configuration from file
            let config = FileSearchConfig::load_from_file(&path_str)
                .context("Failed to load configuration file")?;
                
            return Ok(Some(config));
        }
        
        // No config file specified
        Ok(None)
    }
    
    /// Process command-line arguments, loading from config file if specified
    pub fn process(&self) -> Result<FileSearchConfig> {
        // First convert CLI args to a config
        let mut config = self.to_config();
        
        // If a config file is specified, load and merge it
        if let Some(config_file) = &self.config_file {
            let loaded_config = FileSearchConfig::load_from_file(config_file)
                .context("Failed to load configuration file")?;
            
            // Merge the loaded config with CLI args (CLI args take precedence)
            config = self.merge_with_config(loaded_config);
        }
        
        // Process the save config request if present
        if let Some(_save_path) = &self.save_config_file {
            // Implementation omitted for brevity
        }
        
        Ok(config)
    }
    
    /// Merge CLI arguments with a loaded configuration
    fn merge_with_config(&self, loaded: FileSearchConfig) -> FileSearchConfig {
        let mut merged = self.to_config();
        
        // Only use values from loaded config if not specified in CLI
        if merged.path.is_none() {
            merged.path = loaded.path;
        }
        
        if merged.file_extension.is_none() {
            merged.file_extension = loaded.file_extension;
        }
        
        if merged.file_name.is_none() {
            merged.file_name = loaded.file_name;
        }
        
        // Only override if CLI didn't specify
        if self.quiet {
            merged.show_progress = false;
        } else if !self.quiet {
            // Keep the CLI setting
        } else {
            // Use the loaded setting
            merged.show_progress = loaded.show_progress;
        }
        
        // Only override threads if CLI specified a value
        if let Some(threads) = self.workers {
            merged.thread_count = Some(threads);
        } else {
            merged.thread_count = loaded.thread_count;
        }
        
        merged
    }
    
    /// Save current configuration to a file
    pub fn save_config(&self, config: &FileSearchConfig) -> Result<()> {
        if let Some(_save_path) = &self.save_config_file {
            config.save_to_file(_save_path)?;
            info!("Configuration saved to: {}", _save_path);
        }
        
        Ok(())
    }
} 