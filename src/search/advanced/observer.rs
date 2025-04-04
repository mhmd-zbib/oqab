use std::path::Path;

/// Observer for file search operations
pub trait SearchObserver: Send + Sync {
    /// Called when a file is found
    fn file_found(&self, _file_path: &Path) {}
    
    /// Called when a directory is processed
    fn directory_processed(&self, _dir_path: &Path) {}
    
    /// Return count of files found so far
    fn files_count(&self) -> usize { 0 }
    
    /// Return count of directories processed so far
    fn directories_count(&self) -> usize { 0 }
}

/// A null observer that does nothing
#[derive(Debug, Clone)]
pub struct NullObserver;

impl SearchObserver for NullObserver {
    // Default implementations are used
} 