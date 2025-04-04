use anyhow::{Result, Context};

use crate::commands::Command;
use crate::config::FileSearchConfig;
use crate::search::SearchService;

/// Command for file searching with configurable options
pub struct SearchCommand {
    config: FileSearchConfig,
    search_service: SearchService,
}

impl SearchCommand {
    /// Create a new search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self {
            config,
            search_service: SearchService::new(),
        }
    }
}

impl Command for SearchCommand {
    fn execute(&self) -> Result<()> {
        // Execute the appropriate search based on configuration
        let results = if self.config.advanced_search {
            self.search_service.execute_advanced_search(&self.config)
                .context("Failed to execute advanced search")?
        } else {
            self.search_service.execute_standard_search(&self.config)
                .context("Failed to execute standard search")?
        };
            
        // Display the results
        self.search_service.display_results(&results)
            .context("Failed to display search results")?;
            
        Ok(())
    }
} 