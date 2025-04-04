use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use log::debug;

use crate::core::config::AppConfig;
use crate::filters::{ExtensionFilter, Filter, FilterResult, NameFilter};

/// Search for files matching criteria in a directory
pub fn search_directory(
    dir: &Path,
    config: &AppConfig,
) -> Vec<PathBuf> {
    debug!("Starting standard search in: {}", dir.display());
    
    let mut results = Vec::new();
    let files_count = Arc::new(AtomicUsize::new(0));
    let dirs_count = Arc::new(AtomicUsize::new(0));
    
    // Create filters based on config
    let mut filters: Vec<Box<dyn Filter>> = Vec::new();
    
    if let Some(ref ext) = config.extension {
        filters.push(Box::new(ExtensionFilter::new(ext)));
    }
    
    if let Some(ref name) = config.name {
        filters.push(Box::new(NameFilter::new(name)));
    }
    
    // Recursively walk the directory
    walk_directory(
        dir, 
        &filters, 
        &mut results, 
        files_count.clone(), 
        dirs_count.clone(),
        config.depth,
        0,
    );
    
    debug!(
        "Standard search completed: found {} files, processed {} directories",
        files_count.load(Ordering::Relaxed),
        dirs_count.load(Ordering::Relaxed)
    );
    
    results
}

/// Recursively walk a directory applying filters
fn walk_directory(
    dir: &Path,
    filters: &[Box<dyn Filter>],
    results: &mut Vec<PathBuf>,
    files_count: Arc<AtomicUsize>,
    dirs_count: Arc<AtomicUsize>,
    max_depth: Option<usize>,
    current_depth: usize,
) {
    // Check if we've reached maximum depth
    if let Some(max) = max_depth {
        if current_depth >= max {
            return;
        }
    }
    
    dirs_count.fetch_add(1, Ordering::Relaxed);
    
    // Try to read directory entries
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            debug!("Failed to read directory {}: {}", dir.display(), e);
            return;
        }
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        
        if path.is_dir() {
            walk_directory(
                &path,
                filters,
                results,
                files_count.clone(),
                dirs_count.clone(),
                max_depth,
                current_depth + 1,
            );
        } else if path.is_file() {
            // Apply all filters
            let mut accepted = true;
            
            for filter in filters {
                if filter.filter(&path) != FilterResult::Accept {
                    accepted = false;
                    break;
                }
            }
            
            if accepted {
                results.push(path);
                files_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
} 