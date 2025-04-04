use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use log::{info, warn, debug};
use crate::SearchObserver;
use thiserror::Error;

/// Observer that reports search progress to the console
#[derive(Debug)]
pub struct ProgressReporter {
    files_found: AtomicUsize,
    directories_processed: AtomicUsize,
}

/// Errors that can occur during observation
#[derive(Error, Debug)]
pub enum ObserverError {
    #[error("Failed to update counter: {0}")]
    AtomicError(String),
    
    #[error("Invalid path provided: {0}")]
    PathError(String),
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new() -> Self {
        Self {
            files_found: AtomicUsize::new(0),
            directories_processed: AtomicUsize::new(0),
        }
    }
    
    /// Get the current number of files found
    pub fn files_found(&self) -> usize {
        self.files_found.load(Ordering::Acquire)
    }
    
    /// Get the current number of directories processed
    pub fn directories_processed(&self) -> usize {
        self.directories_processed.load(Ordering::Acquire)
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
        if !file_path.exists() {
            warn!("Attempted to report non-existent file: {:?}", file_path);
            return;
        }
        
        let count = self.files_found.fetch_add(1, Ordering::Release) + 1;
        if count % 10 == 0 {
            info!("Found {} files so far", count);
        }
        debug!("Found: {}", file_path.display());
    }
    
    fn directory_processed(&self, dir_path: &Path) {
        if !dir_path.is_dir() && dir_path.exists() {
            warn!("Attempted to report non-directory path: {:?}", dir_path);
            return;
        }
        
        let count = self.directories_processed.fetch_add(1, Ordering::Release) + 1;
        if count % 10 == 0 {
            info!("Processed {} directories so far", count);
        }
        debug!("Searching: {}", dir_path.display());
    }
    
    fn files_count(&self) -> usize {
        self.files_found.load(Ordering::Acquire)
    }
    
    fn directories_count(&self) -> usize {
        self.directories_processed.load(Ordering::Acquire)
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
    // Using default implementations - all methods are provided by the trait's default implementations
} 