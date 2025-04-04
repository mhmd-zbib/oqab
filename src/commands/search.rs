use anyhow::Result;
use log::{info, debug};
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::commands::Command;
use crate::core::FileSearchConfig;
use crate::core::observer::{TrackingObserver, SearchObserver};
use crate::core::registry::ObserverRegistry;
use crate::utils::search_directory;

/// Command for searching files
pub struct SearchCommand<'a> {
    config: &'a FileSearchConfig,
    start_time: Instant,
}

impl<'a> SearchCommand<'a> {
    /// Create a new search command
    pub fn new(config: &'a FileSearchConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
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
        app_config.show_progress = Some(self.config.show_progress);
        
        // Add size filters
        app_config.min_size = self.config.min_size;
        app_config.max_size = self.config.max_size;
        
        // Add date filters
        app_config.newer_than = self.config.newer_than.clone();
        app_config.older_than = self.config.older_than.clone();
        
        debug!("Size filters: min={:?}, max={:?}", app_config.min_size, app_config.max_size);
        debug!("Date filters: newer={:?}, older={:?}", app_config.newer_than, app_config.older_than);
        
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
        let elapsed = self.start_time.elapsed();
        
        if !files.is_empty() {
            println!("\nFound {} matching file(s):", files.len());
            for file in files {
                println!("  {}", file.display());
            }
            
            // Show performance metrics if not in silent mode
            if self.config.show_progress {
                self.display_performance_metrics(files.len(), elapsed);
            }
        } else {
            println!("\nNo matching files found");
            
            // Even when no files are found, show how long the search took
            if self.config.show_progress {
                self.display_performance_metrics(0, elapsed);
            }
        }
        
        Ok(())
    }
    
    /// Display performance metrics
    fn display_performance_metrics(&self, files_count: usize, elapsed: Duration) {
        let elapsed_secs = elapsed.as_secs_f64();
        let files_per_sec = if elapsed_secs > 0.0 && files_count > 0 {
            files_count as f64 / elapsed_secs
        } else {
            0.0
        };
        
        println!("\nPerformance:");
        println!("  Time taken: {:.2} seconds", elapsed_secs);
        println!("  Files found: {}", files_count);
        println!("  Processing rate: {:.2} files/sec", files_per_sec);
    }
} 