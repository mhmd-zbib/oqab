use std::path::PathBuf;
use anyhow::Result;
use log::info;

use crate::commands::Command;
use crate::config::FileSearchConfig;
use crate::search::FinderFactory;
use crate::{SearchObserver, ObserverRegistry};
use crate::observers::{ProgressReporter, SilentObserver};
use std::sync::Arc;

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
        ProgressReporter::new()
    }
}

impl Command for AdvancedSearchCommand {
    fn execute(&self) -> Result<()> {
        // Create an appropriate observer based on configuration
        let observer = self.create_observer();
        
        // Run the search with the selected path
        let search_path = PathBuf::from(self.config.path.as_deref().unwrap_or("."));
        info!("Searching in directory: {}", search_path.display());
        
        let finder = match &self.config.file_name {
            Some(name) => {
                // Create a name-based finder
                if let Some(ext) = &self.config.file_extension {
                    // Create advanced finder to handle both
                    let ext_finder = crate::search::advanced::OqabFinderFactory::create_combined_finder(
                        name, ext, Arc::new(ObserverRegistry::Progress(observer))
                    );
                    ext_finder.find(&search_path)
                } else {
                    // Just find files with the name pattern
                    let name_finder = crate::search::advanced::OqabFinderFactory::create_name_filter_with_observer(
                        name, Arc::new(ObserverRegistry::Progress(observer))
                    );
                    name_finder.find(&search_path)
                }
            },
            None => {
                // Just use extension filter
                if let Some(ext) = &self.config.file_extension {
                    let ext_finder = crate::search::advanced::OqabFinderFactory::create_extension_finder(ext);
                    ext_finder.find(&search_path)
                } else {
                    anyhow::bail!("No search criteria provided. Please specify either a name pattern or file extension");
                }
            }
        }?;
        
        // Print the found files
        if !finder.is_empty() {
            println!("Found {} matching files:", finder.len());
            for file in finder {
                println!("{}", file.display());
            }
        } else {
            println!("No matching files found");
        }
        
        Ok(())
    }
} 