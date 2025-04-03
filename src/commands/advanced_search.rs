use std::path::{PathBuf, Path};
use log::error;

use anyhow::Result;
use crate::config::FileSearchConfig;
use crate::commands::Command;
use crate::search::{FinderFactory, SearchObserver};
use crate::observers::{ProgressReporter, SilentObserver};

/// Command for advanced file searching with progress reporting
pub struct AdvancedSearchCommand {
    config: FileSearchConfig,
}

impl AdvancedSearchCommand {
    /// Create a new advanced search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self { config }
    }
    
    /// Create an appropriate observer
    fn create_observer(&self) -> Box<dyn SearchObserver + 'static> {
        if self.config.show_progress {
            Box::new(ProgressReporter::new())
        } else {
            Box::new(SilentObserver::new())
        }
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
}

impl Command for AdvancedSearchCommand {
    fn execute(&self) -> Result<()> {
        // Get the search path
        let search_path = self.config.get_path();
        let path = Path::new(&search_path);
        let observer = self.create_observer();
        
        // Create finder using factory based on search criteria
        let finder = if let Some(name) = &self.config.file_name {
            if let Some(ext) = &self.config.file_extension {
                // Find files with both name and extension
                FinderFactory::create_extension_and_name_finder(ext, name, Some(observer))
            } else {
                // Find files with name only
                FinderFactory::create_name_finder_with_observer(name, observer)
            }
        } else if let Some(ext) = &self.config.file_extension {
            // Find files with extension only
            FinderFactory::create_extension_finder_with_observer(ext, observer)
        } else {
            // Use wildcard if neither specified (should be caught by validation)
            FinderFactory::create_extension_finder_with_observer(".*", Box::new(SilentObserver::new()))
        };
        
        // Execute the search
        match finder.find(path) {
            Ok(files) => {
                self.display_results(files)?;
                Ok(())
            }
            Err(e) => {
                error!("Search failed: {}", e);
                Err(anyhow::Error::from(e))
            }
        }
    }
} 