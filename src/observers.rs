use std::path::Path;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::{info, error};
use crate::search::SearchObserver;

/// Observer that reports search progress to the console
pub struct ProgressReporter {
    files_found: AtomicUsize,
    directories_processed: AtomicUsize,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new() -> Self {
        Self {
            files_found: AtomicUsize::new(0),
            directories_processed: AtomicUsize::new(0),
        }
    }
}

impl SearchObserver for ProgressReporter {
    fn on_file_found(&self, file: &Path) {
        let count = self.files_found.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 10 == 0 {
            info!("Found {} files so far", count);
        }
        info!("Found: {}", file.display());
    }
    
    fn on_directory_entry(&self, path: &Path, entries_count: usize) {
        let count = self.directories_processed.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 10 == 0 {
            info!("Processed {} directories so far", count);
        }
        info!("Searching: {} (contains {} entries)", path.display(), entries_count);
    }
    
    fn on_error(&self, error: &io::Error, path: &Path) {
        error!("Error accessing directory {}: {}", path.display(), error);
    }
    
    fn on_search_complete(&self, found_count: usize, dirs_count: usize, duration_ms: u128) {
        let dirs_processed = self.directories_processed.load(Ordering::Relaxed);

        info!("Search completed in {}ms", duration_ms);
        info!("Results:");
        info!("  Directories processed: {}", dirs_processed);
        info!("  Files found: {}", found_count);
        
        if duration_ms > 0 {
            let files_per_second = (found_count as f64) / (duration_ms as f64 / 1000.0);
            let dirs_per_second = (dirs_count as f64) / (duration_ms as f64 / 1000.0);
            info!("  Performance: {:.1} files/sec, {:.1} dirs/sec", files_per_second, dirs_per_second);
        }
    }
}

/// Silent observer that doesn't report any progress
pub struct SilentObserver;

impl SilentObserver {
    /// Create a new silent observer
    pub fn new() -> Self {
        Self
    }
}

impl SearchObserver for SilentObserver {
    fn on_file_found(&self, _file: &Path) {
        // Do nothing - silent implementation
    }
    
    fn on_directory_entry(&self, _dir: &Path, _entries_count: usize) {
        // Do nothing - silent implementation
    }
    
    fn on_error(&self, _error: &io::Error, _path: &Path) {
        // Do nothing - silent implementation
    }
    
    fn on_search_complete(&self, _found_count: usize, _dirs_count: usize, _duration_ms: u128) {
        // Do nothing - silent implementation
    }
} 