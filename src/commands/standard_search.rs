use std::path::{PathBuf, Path};
use anyhow::Result;
use log::error;

use crate::config::FileSearchConfig;
use crate::commands::Command;
use crate::search::FinderFactory;

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
}

impl Command for StandardSearchCommand {
    fn execute(&self) -> Result<()> {
        // Get the search path, with a default of "." (current directory)
        let search_path = self.config.get_path();
        let path = Path::new(&search_path);
        
        // Create finder using factory based on search criteria
        let finder = if let Some(name) = &self.config.file_name {
            if let Some(ext) = &self.config.file_extension {
                // Find files with both name and extension
                FinderFactory::create_name_and_extension_finder(name, ext)
            } else {
                // Find files with name only
                FinderFactory::create_name_finder(name)
            }
        } else if let Some(ext) = &self.config.file_extension {
            // Find files with extension only
            FinderFactory::create_extension_finder(ext)
        } else {
            // Use wildcard if neither specified (should be caught by validation)
            FinderFactory::create_extension_finder(".*")
        };
        
        // Execute the search
        match finder.find(path) {
            Ok(files) => {
                self.display_results(files)?;
                Ok(())
            },
            Err(e) => {
                error!("Failed to search for files: {}", e);
                Err(e)
            }
        }
    }
} 