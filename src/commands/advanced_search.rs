use std::path::{PathBuf, Path};
use anyhow::{Result, Context, bail};
use thiserror::Error;
use log::{info, debug, error};

use crate::commands::Command;
use crate::config::FileSearchConfig;
use crate::ObserverRegistry;
use crate::observers::ProgressReporter;
use std::sync::Arc;

/// Advanced search command errors
#[derive(Error, Debug)]
pub enum AdvancedSearchError {
    #[error("No search criteria provided")]
    NoCriteriaError,
    
    #[error("Invalid search path: {0}")]
    InvalidPathError(String),
    
    #[error("Search execution failed: {0}")]
    SearchExecutionError(String),
}

/// Command for advanced file searching with progress reporting
pub struct AdvancedSearchCommand {
    config: FileSearchConfig,
}

impl AdvancedSearchCommand {
    /// Create a new advanced search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self { config }
    }

    /// Create an appropriate observer based on config
    fn create_observer(&self) -> ProgressReporter {
        let reporter = ProgressReporter::new();
        debug!("Created progress reporter with threading mode: {}", 
            if self.config.thread_count.is_some() { "parallel" } else { "single-threaded" });
        reporter
    }
    
    /// Validate the search configuration
    fn validate_config(&self) -> Result<()> {
        // Check that at least one search criteria is provided
        if self.config.file_name.is_none() && self.config.file_extension.is_none() {
            return Err(AdvancedSearchError::NoCriteriaError.into());
        }
        
        // Check that the search path exists
        let path = Path::new(self.config.get_path());
        if !path.exists() {
            return Err(AdvancedSearchError::InvalidPathError(path.display().to_string()).into());
        }
        
        // Validate that the path is a directory
        if !path.is_dir() {
            return Err(AdvancedSearchError::InvalidPathError(
                format!("Path is not a directory: {}", path.display())
            ).into());
        }
        
        Ok(())
    }
    
    /// Display search results
    fn display_results(&self, files: Vec<PathBuf>) -> Result<()> {
        // Print the found files
        if !files.is_empty() {
            println!("\nFound {} matching files:", files.len());
            for file in files {
                println!("  {}", file.display());
            }
        } else {
            println!("\nNo matching files found");
        }
        
        Ok(())
    }
}

impl Command for AdvancedSearchCommand {
    fn execute(&self) -> Result<()> {
        // First validate the configuration
        self.validate_config()
            .context("Invalid search configuration")?;
        
        // Create an appropriate observer based on configuration
        let observer = self.create_observer();
        
        // Get the search path
        let search_path = PathBuf::from(self.config.get_path());
        info!("Performing advanced search in directory: {}", search_path.display());
        
        // Execute the appropriate finder based on search criteria
        let finder_result = match &self.config.file_name {
            Some(name) => {
                // Create a name-based finder
                if let Some(ext) = &self.config.file_extension {
                    debug!("Searching for files with name '{}' and extension '{}'", name, ext);
                    // Create advanced finder to handle both
                    let ext_finder = crate::search::advanced::OqabFinderFactory::create_combined_finder(
                        name, ext, Arc::new(ObserverRegistry::Progress(observer.clone()))
                    );
                    ext_finder.find(&search_path)
                } else {
                    debug!("Searching for files with name '{}'", name);
                    // Just find files with the name pattern
                    let name_finder = crate::search::advanced::OqabFinderFactory::create_name_filter_with_observer(
                        name, Arc::new(ObserverRegistry::Progress(observer.clone()))
                    );
                    name_finder.find(&search_path)
                }
            },
            None => {
                // Just use extension filter
                if let Some(ext) = &self.config.file_extension {
                    debug!("Searching for files with extension '{}'", ext);
                    let ext_finder = crate::search::advanced::OqabFinderFactory::create_extension_finder(ext);
                    ext_finder.find(&search_path)
                } else {
                    // This case should be caught by validate_config
                    bail!(AdvancedSearchError::NoCriteriaError);
                }
            }
        }.with_context(|| AdvancedSearchError::SearchExecutionError(
            format!("Failed to search in path: {}", search_path.display())
        ))?;
        
        // Process results
        info!("Search completed, found {} files", finder_result.len());
        
        // Display the results
        self.display_results(finder_result)
            .context("Failed to display search results")?;
        
        Ok(())
    }
} 