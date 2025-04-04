use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    core::{
        registry::{FilterRegistry, ObserverRegistry},
        traversal::TraversalStrategy,
        worker::WorkerPool,
    },
    filters::FilterResult,
};

/// Configuration for file finder
#[derive(Debug, Clone)]
pub struct FinderConfig {
    /// Number of threads to use for search
    pub num_threads: usize,
    /// Whether to follow symbolic links
    pub follow_links: bool,
    /// Maximum depth to search
    pub max_depth: Option<usize>,
}

impl Default for FinderConfig {
    fn default() -> Self {
        FinderConfig {
            num_threads: num_cpus::get(),
            follow_links: false,
            max_depth: None,
        }
    }
}

/// Main file finder implementation
pub struct FileFinder {
    config: FinderConfig,
    traversal_strategy: Arc<dyn TraversalStrategy>,
    filter_registry: Arc<FilterRegistry>,
    observer_registry: Arc<ObserverRegistry>,
}

impl FileFinder {
    /// Create a new FileFinder with the given configuration
    pub fn new(
        config: FinderConfig,
        traversal_strategy: Arc<dyn TraversalStrategy>,
        filter_registry: Arc<FilterRegistry>,
        observer_registry: Arc<ObserverRegistry>,
    ) -> Self {
        FileFinder {
            config,
            traversal_strategy,
            filter_registry,
            observer_registry,
        }
    }

    /// Find files in the given directory
    pub fn find(&self, root_dir: &Path) -> Vec<PathBuf> {
        let found_files = Vec::new();
        let traversal = Arc::clone(&self.traversal_strategy);
        let filters = Arc::clone(&self.filter_registry);
        let observers = Arc::clone(&self.observer_registry);
        
        let worker_pool = WorkerPool::new(
            self.config.num_threads,
            
            // Directory consumer
            {
                let traversal = Arc::clone(&traversal);
                let filters = Arc::clone(&filters);
                let observers = Arc::clone(&observers);
                let config = self.config.clone();
                
                move |dir_path| {
                    process_directory(
                        &dir_path,
                        &traversal,
                        &filters,
                        &observers,
                        &config,
                        &mut Vec::new(),
                    );
                }
            },
            
            // File consumer
            {
                let filters = Arc::clone(&filters);
                let observers = Arc::clone(&observers);
                
                move |file_path| {
                    if let FilterResult::Accept = filters.apply_all(&file_path) {
                        observers.notify_file_found(&file_path);
                        // In a real implementation we would collect into found_files
                        // but we need to wait for worker pool termination
                    }
                }
            },
        );
        
        // Process the root directory
        worker_pool.submit_directory(root_dir);
        worker_pool.complete();
        
        // In a real implementation we would collect the results from some shared storage
        // For now, we just return found_files which might be empty
        found_files
    }
}

/// Process a directory during the file search
fn process_directory(
    dir_path: &Path,
    traversal_strategy: &Arc<dyn TraversalStrategy>,
    filter_registry: &Arc<FilterRegistry>,
    observer_registry: &Arc<ObserverRegistry>,
    config: &FinderConfig,
    current_depth: &mut Vec<String>,
) {
    // Check depth limit
    if let Some(max_depth) = config.max_depth {
        if current_depth.len() >= max_depth {
            return;
        }
    }
    
    if !traversal_strategy.should_process_directory(dir_path) {
        return;
    }
    
    observer_registry.notify_directory_processed(dir_path);
    
    // Try to read directory entries
    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        
        if file_type.is_dir() {
            // Push directory name to track depth
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                current_depth.push(dir_name.to_string());
                process_directory(
                    &path, 
                    traversal_strategy, 
                    filter_registry, 
                    observer_registry, 
                    config, 
                    current_depth
                );
                current_depth.pop();
            }
        } else if file_type.is_file() && traversal_strategy.should_process_file(&path) {
            match filter_registry.apply_all(&path) {
                FilterResult::Accept => {
                    observer_registry.notify_file_found(&path);
                }
                _ => {}
            }
        } else if file_type.is_symlink() && config.follow_links {
            // Follow symlinks if enabled
            if let Ok(target) = std::fs::read_link(&path) {
                let target_path = if target.is_absolute() {
                    target
                } else {
                    // Make path relative to the symlink's directory
                    let parent = path.parent().unwrap_or(Path::new(""));
                    parent.join(&target)
                };
                
                if let Ok(metadata) = std::fs::metadata(&target_path) {
                    if metadata.is_dir() {
                        if let Some(dir_name) = target_path.file_name().and_then(|n| n.to_str()) {
                            current_depth.push(dir_name.to_string());
                            process_directory(
                                &target_path,
                                traversal_strategy,
                                filter_registry,
                                observer_registry,
                                config,
                                current_depth,
                            );
                            current_depth.pop();
                        }
                    } else if metadata.is_file() && traversal_strategy.should_process_file(&target_path) {
                        match filter_registry.apply_all(&target_path) {
                            FilterResult::Accept => {
                                observer_registry.notify_file_found(&target_path);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
} 