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
        
        // Create observer with tracking
        let (observer_registry, tracker) = self.create_observer_registry();
        
        // Execute the appropriate search based on configuration
        let results = if self.config.advanced_search {
            // Use the advanced finder (currently using standard for testing)
            info!("Using advanced search mode");
            
            // Create and configure the advanced search
            let builder = crate::core::builder::FileFinderBuilder::new()
                .with_threads(app_config.threads.unwrap_or_else(num_cpus::get))
                .with_observer_registry((*observer_registry).clone())
                .with_traversal_strategy(Box::new(crate::core::traversal::DefaultTraversalStrategy::new(true)));
                
            // Add extension filter if specified
            let mut builder = if let Some(ref ext) = app_config.extension {
                builder.with_filter("extension", crate::filters::ExtensionFilter::new(ext))
            } else {
                builder
            };
            
            // Add name filter if specified
            builder = if let Some(ref name) = app_config.name {
                builder.with_filter("name", crate::filters::NameFilter::new(name))
            } else {
                builder
            };
            
            // Build and run the finder
            let finder = builder.build();
            finder.find(&app_config.root_dir);
            
            // Get results from the tracker
            tracker.get_found_files()
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