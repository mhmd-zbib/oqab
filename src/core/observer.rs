use std::{
    path::Path,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
    sync::{Mutex, MutexGuard},
    any::Any,
};
use log::warn;
use anyhow::Result;
pub trait SearchObserver: Send + Sync {
    // Observer for file search operations
    fn file_found(&self, file_path: &Path);
    fn directory_processed(&self, dir_path: &Path);
    fn files_count(&self) -> usize;
    fn directories_count(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
}
#[derive(Debug)]
pub struct NullObserver;
impl SearchObserver for NullObserver {
    fn file_found(&self, _file_path: &Path) {}
    fn directory_processed(&self, _dir_path: &Path) {}
    fn files_count(&self) -> usize { 0 }
    fn directories_count(&self) -> usize { 0 }
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for NullObserver {
    fn clone(&self) -> Self {
        Self
    }
}
#[derive(Debug)]
pub struct ProgressReporter {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
    start_time: Instant,
}
impl ProgressReporter {
    pub fn new() -> Self {
        ProgressReporter {
            files_count: AtomicUsize::new(0),
            dirs_count: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }
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
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for ProgressReporter {
    fn clone(&self) -> Self {
        let new_reporter = ProgressReporter::new();
        let files_count = self.files_count();
        if files_count > 0 {
            new_reporter.files_count.store(files_count, Ordering::Relaxed);
        }
        let dirs_count = self.directories_count();
        if dirs_count > 0 {
            new_reporter.dirs_count.store(dirs_count, Ordering::Relaxed);
        }
        new_reporter
    }
}
#[derive(Debug)]
pub struct SilentObserver {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
}
impl SilentObserver {
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
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for SilentObserver {
    fn clone(&self) -> Self {
        SilentObserver {
            files_count: AtomicUsize::new(self.files_count()),
            dirs_count: AtomicUsize::new(self.directories_count()),
        }
    }
}
#[derive(Debug)]
pub struct TrackingObserver {
    files_count: AtomicUsize,
    dirs_count: AtomicUsize,
    found_files: Mutex<Vec<PathBuf>>,
}
impl TrackingObserver {
    pub fn new() -> Self {
        TrackingObserver {
            files_count: AtomicUsize::new(0),
            dirs_count: AtomicUsize::new(0),
            found_files: Mutex::new(Vec::new()),
        }
    }
    pub fn lock_found_files(&self) -> Result<MutexGuard<'_, Vec<PathBuf>>> {
        self.found_files.lock()
            .map_err(|_e| anyhow::anyhow!("Failed to acquire lock on found_files: poisoned lock"))
    }
    #[deprecated(
        since = "0.2.0",
        note = "This method is inefficient. Use lock_found_files() instead for better performance."
    )]
    pub fn get_found_files(&self) -> Vec<PathBuf> {
        match self.found_files.lock() {
            Ok(files) => files.clone(),
            Err(_e) => {
                warn!("Failed to acquire lock for get_found_files, returning empty vector");
                Vec::new()
            }
        }
    }
    pub fn merge_from(&self, other: &TrackingObserver) -> Result<()> {
        let other_files = other.lock_found_files()?;
        let mut my_files = self.lock_found_files()?;
        my_files.reserve(other_files.len());
        my_files.extend_from_slice(&other_files);
        let other_files_count = other.files_count();
        if other_files_count > 0 {
            self.files_count.fetch_add(other_files_count, Ordering::Relaxed);
        }
        let other_dirs_count = other.directories_count();
        if other_dirs_count > 0 {
            self.dirs_count.fetch_add(other_dirs_count, Ordering::Relaxed);
        }
        Ok(())
    }
}
impl Default for TrackingObserver {
    fn default() -> Self {
        Self::new()
    }
}
impl SearchObserver for TrackingObserver {
    fn file_found(&self, file_path: &Path) {
        self.files_count.fetch_add(1, Ordering::Relaxed);
        match self.found_files.lock() {
            Ok(mut files) => {
                files.push(file_path.to_path_buf());
            },
            Err(_e) => {
                warn!("Failed to store found file {}: poisoned lock", file_path.display());
            }
        }
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
    fn as_any(&self) -> &dyn Any { self }
}
impl Clone for TrackingObserver {
    fn clone(&self) -> Self {
        let new_observer = TrackingObserver::new();
        let files_count = self.files_count();
        if files_count > 0 {
            new_observer.files_count.store(files_count, Ordering::Relaxed);
        }
        let dirs_count = self.directories_count();
        if dirs_count > 0 {
            new_observer.dirs_count.store(dirs_count, Ordering::Relaxed);
        }
        new_observer
    }
}

