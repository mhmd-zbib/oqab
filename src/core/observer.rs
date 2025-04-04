use std::{
    path::Path,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
    sync::Mutex,
};

/// Observer for file search operations
pub trait SearchObserver: Send + Sync {
    /// Called when a file is found
    fn file_found(&self, file_path: &Path);
    
    /// Called when a directory is processed
    fn directory_processed(&self, dir_path: &Path);
    
    /// Return count of files found so far
    fn files_count(&self) -> usize;
    
    /// Return count of directories processed so far
    fn directories_count(&self) -> usize;
}

/// A null observer that does nothing
#[derive(Debug, Clone)]
pub struct NullObserver;

impl SearchObserver for NullObserver {
    fn file_found(&self, _file_path: &Path) {}
    
    fn directory_processed(&self, _dir_path: &Path) {}
    
    fn files_count(&self) -> usize { 0 }
    
    fn directories_count(&self) -> usize { 0 }
}

/// A progress reporting observer
#[derive(Debug)]
pub struct ProgressReporter {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
    start_time: Instant,
}

impl ProgressReporter {
    /// Create a new ProgressReporter
    pub fn new() -> Self {
        ProgressReporter {
            files_count: AtomicUsize::new(0),
            dirs_count: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }
    
    /// Get elapsed time since search started
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchObserver for ProgressReporter {
    fn file_found(&self, file_path: &Path) {
        let count = self.files_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 100 == 0 {
            println!("Found {} files so far... (latest: {})", 
                count, file_path.display());
        }
    }
    
    fn directory_processed(&self, dir_path: &Path) {
        let count = self.dirs_count.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 50 == 0 {
            println!("Processed {} directories so far... (latest: {})", 
                count, dir_path.display());
        }
    }
    
    fn files_count(&self) -> usize {
        self.files_count.load(Ordering::Relaxed)
    }
    
    fn directories_count(&self) -> usize {
        self.dirs_count.load(Ordering::Relaxed)
    }
}

/// A silent observer that only counts
#[derive(Debug)]
pub struct SilentObserver {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
}

impl SilentObserver {
    /// Create a new SilentObserver
    pub fn new() -> Self {
        SilentObserver {
            files_count: AtomicUsize::new(0),
            dirs_count: AtomicUsize::new(0),
        }
    }
}

impl Default for SilentObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchObserver for SilentObserver {
    fn file_found(&self, _file_path: &Path) {
        self.files_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn directory_processed(&self, _dir_path: &Path) {
        self.dirs_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn files_count(&self) -> usize {
        self.files_count.load(Ordering::Relaxed)
    }
    
    fn directories_count(&self) -> usize {
        self.dirs_count.load(Ordering::Relaxed)
    }
}

/// An observer that tracks found files in a vector
#[derive(Debug)]
pub struct TrackingObserver {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
    found_files: Mutex<Vec<PathBuf>>,
}

impl TrackingObserver {
    /// Create a new TrackingObserver
    pub fn new() -> Self {
        TrackingObserver {
            files_count: AtomicUsize::new(0),
            dirs_count: AtomicUsize::new(0),
            found_files: Mutex::new(Vec::new()),
        }
    }
    
    /// Get a copy of the found files
    pub fn get_found_files(&self) -> Vec<PathBuf> {
        let files = self.found_files.lock().unwrap();
        files.clone()
    }
}

impl Default for TrackingObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchObserver for TrackingObserver {
    fn file_found(&self, file_path: &Path) {
        // Count the file
        self.files_count.fetch_add(1, Ordering::Relaxed);
        
        // Store the path
        let mut files = self.found_files.lock().unwrap();
        files.push(file_path.to_path_buf());
    }
    
    fn directory_processed(&self, _dir_path: &Path) {
        self.dirs_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn files_count(&self) -> usize {
        self.files_count.load(Ordering::Relaxed)
    }
    
    fn directories_count(&self) -> usize {
        self.dirs_count.load(Ordering::Relaxed)
    }
}