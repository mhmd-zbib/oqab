use anyhow::{Result, Context};

use crate::config::FileSearchConfig;
use crate::commands::Command;
use crate::search::SearchService;

/// Command for standard file searching without advanced features
pub struct StandardSearchCommand {
    config: FileSearchConfig,
    search_service: SearchService,
}

impl StandardSearchCommand {
    /// Create a new standard search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self {
            config,
            search_service: SearchService::new(),
        }
    }
}

impl Command for StandardSearchCommand {
    fn execute(&self) -> Result<()> {
        // Execute the standard search
        let results = self.search_service.execute_standard_search(&self.config)
            .context("Failed to execute standard search")?;
            
        // Display the results
        self.search_service.display_results(&results)
            .context("Failed to display search results")?;
            
        Ok(())
    }
} 