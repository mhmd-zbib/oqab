use std::path::{PathBuf, Path};
use anyhow::{Result, Context, anyhow};
use thiserror::Error;
use log::{error, info, debug};

use crate::config::FileSearchConfig;
use crate::commands::Command;
use crate::search::FinderFactory;

/// Errors specific to standard search commands
#[derive(Error, Debug)]
pub enum SearchCommandError {
    #[error("No search criteria specified")]
    NoCriteriaError,
    
    #[error("Failed to find files: {0}")]
    FinderError(String),
    
    #[error("Invalid search path: {0}")]
    PathError(String),
}

/// Command for standard file searching without advanced features
pub struct StandardSearchCommand {
    config: FileSearchConfig,
}

impl StandardSearchCommand {
    /// Create a new standard search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self { config }
    }
    
    /// Display search results
    fn display_results(&self, files: Vec<PathBuf>) -> Result<()> {
        // Print results
        println!("\nFound {} file(s):", files.len());
        
        for file in &files {
            println!("  {}", file.display());
        }
        
        Ok(())
    }
    
    /// Validate search configuration
    fn validate_config(&self) -> Result<()> {
        // Check if we have at least one search criteria
        if self.config.file_extension.is_none() && self.config.file_name.is_none() {
            return Err(SearchCommandError::NoCriteriaError.into());
        }
        
        // Check if search path exists
        let path = Path::new(self.config.get_path());
        if !path.exists() {
            return Err(SearchCommandError::PathError(path.display().to_string()).into());
        }
        
        Ok(())
    }
}

impl Command for StandardSearchCommand {
    fn execute(&self) -> Result<()> {
        // First validate the configuration
        self.validate_config()
            .context("Invalid search configuration")?;
        
        // Get the search path
        let search_path = self.config.get_path();
        let path = Path::new(search_path);
        
        debug!("Executing standard search in path: {}", path.display());
        
        // Create finder using factory based on search criteria
        let finder = if let Some(name) = &self.config.file_name {
            if let Some(ext) = &self.config.file_extension {
                // Find files with both name and extension
                info!("Searching for files with name '{}' and extension '{}'", name, ext);
                FinderFactory::create_name_and_extension_finder(name, ext)
            } else {
                // Find files with name only
                info!("Searching for files with name '{}'", name);
                FinderFactory::create_name_finder(name)
            }
        } else if let Some(ext) = &self.config.file_extension {
            // Find files with extension only
            info!("Searching for files with extension '{}'", ext);
            FinderFactory::create_extension_finder(ext)
        } else {
            // This case should be caught by validate_config
            return Err(anyhow!("No search criteria specified"));
        };
        
        // Execute the search
        match finder.find(path) {
            Ok(files) => {
                info!("Search completed, found {} files", files.len());
                self.display_results(files)
                    .context("Failed to display search results")?;
                Ok(())
            },
            Err(e) => {
                let err_msg = format!("Failed to search for files: {}", e);
                error!("{}", err_msg);
                Err(anyhow!(SearchCommandError::FinderError(err_msg)))
            }
        }
    }
} 