use std::path::{PathBuf, Path};
use anyhow::{Result, Context};
use log::info;

use crate::commands::Command;
use crate::config::FileSearchConfig;
use crate::search::SearchService;

/// Command for advanced file searching with progress reporting
pub struct AdvancedSearchCommand {
    config: FileSearchConfig,
    search_service: SearchService,
}

impl AdvancedSearchCommand {
    /// Create a new advanced search command
    pub fn new(config: FileSearchConfig) -> Self {
        Self {
            config,
            search_service: SearchService::new(),
        }
    }
}

impl Command for AdvancedSearchCommand {
    fn execute(&self) -> Result<()> {
        // Execute the advanced search
        let results = self.search_service.execute_advanced_search(&self.config)
            .context("Failed to execute advanced search")?;
            
        // Display the results
        self.search_service.display_results(&results)
            .context("Failed to display search results")?;
            
        Ok(())
    }
} 