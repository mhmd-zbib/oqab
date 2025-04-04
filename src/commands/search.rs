use anyhow::Result;
use log::{info, debug};
use std::sync::Arc;
use std::path::PathBuf;

use crate::commands::Command;
use crate::core::FileSearchConfig;
use crate::core::observer::{TrackingObserver, SearchObserver};
use crate::core::registry::ObserverRegistry;
use crate::utils::search_directory;

/// Command for searching files
pub struct SearchCommand<'a> {
    config: &'a FileSearchConfig,
}

impl<'a> SearchCommand<'a> {
    /// Create a new search command
    pub fn new(config: &'a FileSearchConfig) -> Self {
        Self {
            config,
        }
    }

    /// Convert FileSearchConfig to AppConfig
    fn create_app_config(&self) -> crate::core::AppConfig {
        let mut app_config = crate::core::AppConfig::default();
        
        app_config.root_dir = std::path::PathBuf::from(self.config.get_path());
        
        debug!("Search path: {:?}", app_config.root_dir);
        debug!("Extension: {:?}", self.config.file_extension);
        debug!("Name: {:?}", self.config.file_name);
        
        app_config.extension = self.config.file_extension.clone();
        app_config.name = self.config.file_name.clone();
        app_config.threads = self.config.thread_count;
        app_config.follow_links = Some(self.config.follow_symlinks);
        
        app_config
    }
    
    /// Get a configured observer registry with tracking
    fn create_observer_registry(&self) -> (Arc<ObserverRegistry>, Arc<TrackingObserver>) {
        let registry = Arc::new(ObserverRegistry::new());
        let tracker = Arc::new(TrackingObserver::new());
        
        // Register the tracker as an observer
        registry.register_arc(Arc::clone(&tracker) as Arc<dyn SearchObserver>);
        
        (registry, tracker)
    }
}

impl<'a> Command for SearchCommand<'a> {
    fn execute(&self) -> Result<()> {
        // Convert to app config
        let app_config = self.create_app_config();
        
        // Create observer with tracking - currently unused but kept for future implementation
        let (_observer_registry, _tracker) = self.create_observer_registry();
        
        debug!("Created observer registry");
        
        // Execute the appropriate search based on configuration
        let results = if self.config.advanced_search {
            // Use the advanced finder
            info!("Using advanced search mode");
            
            // For now, just use the standard search for both modes
            // In a real implementation, we'd use the advanced finder
            debug!("Using standard search implementation for advanced mode");
            search_directory(&app_config.root_dir, &app_config)
        } else {
            // Use the standard search
            info!("Using standard search mode");
            search_directory(&app_config.root_dir, &app_config)
        };
            
        // Display the results
        self.display_results(&results)?;
            
        Ok(())
    }
}

impl<'a> SearchCommand<'a> {
    /// Format and display search results
    fn display_results(&self, files: &[PathBuf]) -> Result<()> {
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