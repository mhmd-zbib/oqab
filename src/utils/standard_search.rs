use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use log::debug;

use crate::core::config::AppConfig;
use crate::filters::{
    ExtensionFilter, 
    Filter, 
    FilterResult, 
    NameFilter, 
    SizeFilter,
    date::DateFilter
};

/// Search statistics for performance tracking
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total time elapsed during search
    pub elapsed_ms: u128,
    /// Number of files found
    pub files_found: usize,
    /// Number of directories processed
    pub dirs_processed: usize,
    /// Number of files processed
    pub files_processed: usize,
}

/// Search for files matching criteria in a directory
pub fn search_directory(
    dir: &Path,
    config: &AppConfig,
) -> Vec<PathBuf> {
    let start_time = Instant::now();
    debug!("Starting standard search in: {}", dir.display());
    
    let mut results = Vec::new();
    let files_count = Arc::new(AtomicUsize::new(0));
    let dirs_count = Arc::new(AtomicUsize::new(0));
    let processed_files_count = Arc::new(AtomicUsize::new(0));
    
    // Create filters based on config
    let mut filters: Vec<Box<dyn Filter>> = Vec::new();
    
    if let Some(ref ext) = config.extension {
        filters.push(Box::new(ExtensionFilter::new(ext)));
    }
    
    if let Some(ref name) = config.name {
        filters.push(Box::new(NameFilter::new(name)));
    }
    
    // Add size filters if specified
    if let (Some(min), Some(max)) = (config.min_size, config.max_size) {
        filters.push(Box::new(SizeFilter::range(min, max)));
    } else {
        if let Some(min) = config.min_size {
            filters.push(Box::new(SizeFilter::min(min)));
        }
        
        if let Some(max) = config.max_size {
            filters.push(Box::new(SizeFilter::max(max)));
        }
    }
    
    // Add date filters if specified
    if let Some(ref newer_than) = config.newer_than {
        if let Ok(filter) = DateFilter::newer_than(newer_than) {
            filters.push(Box::new(filter));
        }
    }
    
    if let Some(ref older_than) = config.older_than {
        if let Ok(filter) = DateFilter::older_than(older_than) {
            filters.push(Box::new(filter));
        }
    }
    
    // Recursively walk the directory
    walk_directory(
        dir, 
        &filters, 
        &mut results, 
        files_count.clone(), 
        dirs_count.clone(),
        processed_files_count.clone(),
        config.depth,
        0,
    );
    
    let elapsed = start_time.elapsed();
    let stats = SearchStats {
        elapsed_ms: elapsed.as_millis(),
        files_found: files_count.load(Ordering::Relaxed),
        dirs_processed: dirs_count.load(Ordering::Relaxed),
        files_processed: processed_files_count.load(Ordering::Relaxed),
    };
    
    debug!(
        "Standard search completed in {:.2}s: found {} files, processed {} directories and {} files",
        elapsed.as_secs_f64(),
        stats.files_found,
        stats.dirs_processed,
        stats.files_processed
    );
    
    // If not in silent mode, print performance stats
    if config.show_progress.unwrap_or(true) {
        let files_per_sec = if elapsed.as_secs_f64() > 0.0 {
            stats.files_processed as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        debug!(
            "Performance: {:.2} files/sec, total time: {:.2}s",
            files_per_sec,
            elapsed.as_secs_f64()
        );
    }
    
    results
}

/// Recursively walk a directory applying filters
fn walk_directory(
    dir: &Path,
    filters: &[Box<dyn Filter>],
    results: &mut Vec<PathBuf>,
    files_count: Arc<AtomicUsize>,
    dirs_count: Arc<AtomicUsize>,
    processed_files_count: Arc<AtomicUsize>,
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
                processed_files_count.clone(),
                max_depth,
                current_depth + 1,
            );
        } else if path.is_file() {
            processed_files_count.fetch_add(1, Ordering::Relaxed);
            
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