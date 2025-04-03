use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::info;
use crate::SearchObserver;

/// Observer that reports search progress to the console
#[derive(Debug)]
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

impl Clone for ProgressReporter {
    fn clone(&self) -> Self {
        Self {
            files_found: AtomicUsize::new(self.files_found.load(Ordering::Relaxed)),
            directories_processed: AtomicUsize::new(self.directories_processed.load(Ordering::Relaxed)),
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchObserver for ProgressReporter {
    fn file_found(&self, file_path: &Path) {
        let count = self.files_found.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 10 == 0 {
            info!("Found {} files so far", count);
        }
        info!("Found: {}", file_path.display());
    }
    
    fn directory_processed(&self, dir_path: &Path) {
        let count = self.directories_processed.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 10 == 0 {
            info!("Processed {} directories so far", count);
        }
        info!("Searching: {}", dir_path.display());
    }
    
    fn files_count(&self) -> usize {
        self.files_found.load(Ordering::Relaxed)
    }
    
    fn directories_count(&self) -> usize {
        self.directories_processed.load(Ordering::Relaxed)
    }
}

/// Silent observer that doesn't report any progress
#[derive(Debug, Clone)]
pub struct SilentObserver;

impl SilentObserver {
    /// Create a new silent observer
    pub fn new() -> Self {
        Self
    }
}

impl Default for SilentObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchObserver for SilentObserver {
    // Using default implementations - all methods removed since they are provided by default implementations
} 