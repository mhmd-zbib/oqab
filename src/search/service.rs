use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use thiserror::Error;
use log::{info, debug};
use std::sync::Arc;

use crate::config::FileSearchConfig;
use crate::search::FinderFactory;
use crate::ObserverRegistry;
use crate::observers::ProgressReporter;

/// Errors that can occur during search operations
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("No search criteria provided")]
    NoCriteriaError,
    
    #[error("Invalid search path: {0}")]
    InvalidPathError(String),
    
    #[error("Search execution failed: {0}")]
    ExecutionError(String),
}

/// Service that encapsulates common search functionality
pub struct SearchService;

impl SearchService {
    /// Create a new search service
    pub fn new() -> Self {
        Self
    }
    
    /// Validate the search configuration
    pub fn validate_config(&self, config: &FileSearchConfig) -> Result<()> {
        // Check that at least one search criteria is provided
        if config.file_name.is_none() && config.file_extension.is_none() {
            return Err(SearchError::NoCriteriaError.into());
        }
        
        // Check that the search path exists
        let path = Path::new(config.get_path());
        if !path.exists() {
            return Err(SearchError::InvalidPathError(path.display().to_string()).into());
        }
        
        // Validate that the path is a directory
        if !path.is_dir() {
            return Err(SearchError::InvalidPathError(
                format!("Path is not a directory: {}", path.display())
            ).into());
        }
        
        Ok(())
    }
    
    /// Execute a standard search based on the configuration
    pub fn execute_standard_search(&self, config: &FileSearchConfig) -> Result<Vec<PathBuf>> {
        // Validate the configuration
        self.validate_config(config)?;
        
        // Get the search path
        let search_path = Path::new(config.get_path());
        debug!("Executing standard search in path: {}", search_path.display());
        
        // Create finder using factory based on search criteria
        let finder = if let Some(name) = &config.file_name {
            if let Some(ext) = &config.file_extension {
                // Find files with both name and extension
                info!("Searching for files with name '{}' and extension '{}'", name, ext);
                FinderFactory::create_combined_finder(name, ext)
            } else {
                // Find files with name only
                info!("Searching for files with name '{}'", name);
                FinderFactory::create_name_finder(name)
            }
        } else if let Some(ext) = &config.file_extension {
            // Find files with extension only
            info!("Searching for files with extension '{}'", ext);
            FinderFactory::create_extension_finder(ext)
        } else {
            // This case should be caught by validate_config
            bail!(SearchError::NoCriteriaError);
        };
        
        // Execute the search
        finder.find(search_path)
            .with_context(|| SearchError::ExecutionError(
                format!("Failed to search in path: {}", search_path.display())
            ))
    }
    
    /// Execute an advanced search with observers based on the configuration
    pub fn execute_advanced_search(&self, config: &FileSearchConfig) -> Result<Vec<PathBuf>> {
        // Validate the configuration
        self.validate_config(config)?;
        
        // Create progress reporter
        let observer = ProgressReporter::new();
        debug!("Created progress reporter for advanced search");
        
        // Get the search path
        let search_path = PathBuf::from(config.get_path());
        info!("Performing advanced search in directory: {}", search_path.display());
        
        // Execute the appropriate finder based on search criteria
        let finder_result = match &config.file_name {
            Some(name) => {
                // Create a name-based finder
                if let Some(ext) = &config.file_extension {
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
                if let Some(ext) = &config.file_extension {
                    debug!("Searching for files with extension '{}'", ext);
                    let ext_finder = crate::search::advanced::OqabFinderFactory::create_extension_finder(ext);
                    ext_finder.find(&search_path)
                } else {
                    // This case should be caught by validate_config
                    bail!(SearchError::NoCriteriaError);
                }
            }
        }.with_context(|| SearchError::ExecutionError(
            format!("Failed to search in path: {}", search_path.display())
        ))?;
        
        // Process results
        info!("Search completed, found {} files", finder_result.len());
        
        Ok(finder_result)
    }
    
    /// Format and display search results
    pub fn display_results(&self, files: &[PathBuf]) -> Result<()> {
        if !files.is_empty() {
            println!("\nFound {} matching file(s):", files.len());
            for file in files {
                println!("  {}", file.display());
            }
        } else {
            println!("\nNo matching files found");
        }
        
        Ok(())
    }
} 