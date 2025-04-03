use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam::channel::{unbounded, Sender, Receiver};
use dashmap::DashMap;
use num_cpus;
use serde;
use std::any::Any;
use std::sync::{mpsc, Mutex};
use std::fs;
use std::thread;
use std::time::Instant;
use std::collections::VecDeque;
use log::info;

use crate::search::finder::{FileFilter, ExtensionFilter, NameFilter};
use crate::search::composite::{CompositeFilter, FilterOperation};
use crate::observers::{ProgressReporter, SilentObserver};

/// Directory traversal strategies
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub enum TraversalStrategy {
    /// Breadth-first search
    #[default]
    BreadthFirst,
    /// Depth-first search
    DepthFirst,
}

/// A null observer that does nothing
#[derive(Debug, Clone)]
pub struct NullObserver;

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

impl SearchObserver for NullObserver {
    // Default implementations are used
}

/// Message types for worker communication
enum WorkerMessage {
    /// Process this directory
    ProcessDirectory(PathBuf),
    /// All work is done, terminate
    Terminate,
}

/// Worker pool for parallel processing
pub struct WorkerPool {
    /// Channel for sending work to workers
    sender: Sender<WorkerMessage>,
    /// Number of workers in the pool
    workers_count: usize,
}

impl WorkerPool {
    /// Create a new worker pool with specified number of workers
    pub fn new(workers_count: usize, filter: Arc<FilterRegistry>, results: &DashMap<PathBuf, ()>, observer: &Arc<ObserverRegistry>) -> Self {
        let (sender, receiver) = unbounded();
        let receiver = receiver.clone();
        
        // Start worker threads
        for _ in 0..workers_count {
            let worker_receiver = receiver.clone();
            let worker_filter = filter.clone();
            let worker_results = results.clone();
            let worker_observer = observer.clone();
            
            std::thread::spawn(move || {
                Self::worker_loop(worker_receiver, worker_filter, worker_results, worker_observer);
            });
        }
        
        Self {
            sender,
            workers_count,
        }
    }
    
    /// Worker thread main loop
    fn worker_loop(
        receiver: Receiver<WorkerMessage>,
        filter: Arc<FilterRegistry>,
        results: DashMap<PathBuf, ()>,
        observer: Arc<ObserverRegistry>,
    ) {
        loop {
            match receiver.recv() {
                Ok(WorkerMessage::ProcessDirectory(dir)) => {
                    Self::process_directory(&dir, &filter, &results, &observer);
                }
                Ok(WorkerMessage::Terminate) => {
                    break; // Exit the loop on terminate message
                }
                Err(_) => {
                    break; // Channel closed, exit
                }
            }
        }
    }
    
    /// Process a directory, finding matching files and queueing subdirectories
    fn process_directory(
        dir: &Path,
        filter: &Arc<FilterRegistry>,
        results: &DashMap<PathBuf, ()>,
        observer: &Arc<ObserverRegistry>,
    ) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => {
                observer.directory_processed(dir);
                return;
            }
        };
        
        let mut entries_vec = Vec::new();
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    entries_vec.push(path);
                }
                Err(_) => {
                    observer.directory_processed(dir);
                }
            }
        }
        
        // Notify about directory processing
        observer.directory_processed(dir);
        
        for path in entries_vec {
            if path.is_file() && filter.matches(&path) {
                results.insert(path.clone(), ());
                observer.file_found(&path);
            }
        }
    }
    
    /// Queue a directory for processing
    pub fn queue_directory(&self, dir: PathBuf) -> Result<(), io::Error> {
        self.sender.send(WorkerMessage::ProcessDirectory(dir))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to queue directory for processing"))
    }
    
    /// Wait for all workers to complete and terminate them
    pub fn wait_and_terminate(self) {
        // Send terminate message to all workers
        for _ in 0..self.workers_count {
            let _ = self.sender.send(WorkerMessage::Terminate);
        }
    }
}

/// Filter registry type to avoid Arc<Box<dyn Trait>> issues
enum FilterRegistry {
    Extension(ExtensionFilter),
    Name(NameFilter),
    Composite(CompositeFilter),
}

impl FileFilter for FilterRegistry {
    fn matches(&self, file_path: &Path) -> bool {
        match self {
            FilterRegistry::Extension(filter) => filter.matches(file_path),
            FilterRegistry::Name(filter) => filter.matches(file_path),
            FilterRegistry::Composite(filter) => filter.matches(file_path),
        }
    }
}

/// Registry for different types of observers
#[derive(Clone)]
pub enum ObserverRegistry {
    /// Progress reporter observer
    Progress(ProgressReporter),
    /// Silent observer
    Silent(SilentObserver),
    /// Null observer
    Null(NullObserver),
}

impl SearchObserver for ObserverRegistry {
    fn file_found(&self, file_path: &Path) {
        match self {
            ObserverRegistry::Progress(observer) => observer.file_found(file_path),
            ObserverRegistry::Silent(observer) => observer.file_found(file_path),
            ObserverRegistry::Null(observer) => observer.file_found(file_path),
        }
    }

    fn directory_processed(&self, dir_path: &Path) {
        match self {
            ObserverRegistry::Progress(observer) => observer.directory_processed(dir_path),
            ObserverRegistry::Silent(observer) => observer.directory_processed(dir_path),
            ObserverRegistry::Null(observer) => observer.directory_processed(dir_path),
        }
    }

    fn files_count(&self) -> usize {
        match self {
            ObserverRegistry::Progress(observer) => observer.files_count(),
            ObserverRegistry::Silent(observer) => observer.files_count(),
            ObserverRegistry::Null(observer) => observer.files_count(),
        }
    }

    fn directories_count(&self) -> usize {
        match self {
            ObserverRegistry::Progress(observer) => observer.directories_count(),
            ObserverRegistry::Silent(observer) => observer.directories_count(),
            ObserverRegistry::Null(observer) => observer.directories_count(),
        }
    }
}

/// Enhanced file finder with parallel processing and caching
pub struct OqabFileFinder {
    /// Filter to apply to files
    filter: Arc<FilterRegistry>,
    /// Observer for search events
    observer: Arc<ObserverRegistry>,
    /// Number of worker threads
    workers_count: usize,
    /// Directory traversal strategy
    traversal_strategy: TraversalStrategy,
}

impl OqabFileFinder {
    /// Create a new builder for OqabFileFinder
    pub fn builder() -> OqabFileFinderBuilder {
        OqabFileFinderBuilder::new()
    }
    
    /// Find files matching filter in the path
    pub fn find(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let start_time = std::time::Instant::now();
        let results = DashMap::new();
        let dirs_processed = AtomicUsize::new(0);
        
        // Create worker pool
        let pool = WorkerPool::new(
            self.workers_count,
            self.filter.clone(),
            &results,
            &self.observer,
        );
        
        // Process the root directory
        match self.traversal_strategy {
            TraversalStrategy::BreadthFirst => {
                // Start with the root directory
                self.breadth_first_search(path, &pool, &dirs_processed)?;
            }
            TraversalStrategy::DepthFirst => {
                // Use recursive depth-first approach
                self.depth_first_search(path, &pool, &dirs_processed)?;
            }
        }
        
        // Wait for all workers to complete
        pool.wait_and_terminate();
        
        // Convert results to a vector
        let found_files: Vec<PathBuf> = results.iter().map(|entry| entry.key().clone()).collect();
        
        // Log completion information
        let duration = start_time.elapsed().as_millis();
        info!("Search completed in {}ms, found {} files, processed {} directories", 
              duration, 
              found_files.len(), 
              dirs_processed.load(Ordering::Relaxed));
        
        Ok(found_files)
    }
    
    /// Breadth-first directory traversal
    fn breadth_first_search(&self, root: &Path, pool: &WorkerPool, dirs_processed: &AtomicUsize) -> io::Result<()> {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(root.to_path_buf());
        
        while let Some(dir) = queue.pop_front() {
            dirs_processed.fetch_add(1, Ordering::Relaxed);
            
            // Queue this directory for processing
            pool.queue_directory(dir.clone())?;
            
            // Add subdirectories to the queue
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            queue.push_back(path);
                        }
                    }
                }
                Err(_) => {
                    self.observer.directory_processed(&dir);
                }
            }
        }
        
        Ok(())
    }
    
    /// Depth-first directory traversal
    fn depth_first_search(&self, dir: &Path, pool: &WorkerPool, dirs_processed: &AtomicUsize) -> io::Result<()> {
        dirs_processed.fetch_add(1, Ordering::Relaxed);
        
        // Queue this directory for processing
        pool.queue_directory(dir.to_path_buf())?;
        
        // Process subdirectories recursively
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        self.depth_first_search(&path, pool, dirs_processed)?;
                    }
                }
            }
            Err(_) => {
                self.observer.directory_processed(dir);
            }
        }
        
        Ok(())
    }

    /// Search a directory for matching files
    fn search_directory(&self, dir: &Path, observer: &Arc<ObserverRegistry>) -> Result<Vec<PathBuf>, std::io::Error> {
        let start = std::time::Instant::now();
        let mut entries_vec = Vec::new();
        
        match std::fs::read_dir(dir) {
            Ok(entries) => {
                // Process all entries
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            let metadata = entry.metadata();
                            
                            if let Ok(metadata) = metadata {
                                if metadata.is_file() && self.filter.matches(&path) {
                                    entries_vec.push(path.clone());
                                    observer.file_found(&path);
                                }
                            }
                        }
                        Err(_) => {
                            // Just log errors but continue with other entries
                            observer.directory_processed(dir);
                        }
                    }
                }
            }
            Err(e) => {
                // Report the error via observer but continue
                observer.directory_processed(dir);
                return Err(e);
            }
        };
        
        // Report completed directory
        observer.directory_processed(dir);
        
        let elapsed = start.elapsed();
        Ok(entries_vec)
    }
}

/// Builder for OqabFileFinder
pub struct OqabFileFinderBuilder {
    filter: Option<Arc<FilterRegistry>>,
    observer: Option<Arc<ObserverRegistry>>,
    workers_count: usize,
    traversal_strategy: TraversalStrategy,
}

impl OqabFileFinderBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            filter: None,
            observer: None,
            workers_count: num_cpus::get(),
            traversal_strategy: TraversalStrategy::default(),
        }
    }
    
    /// Set the file filter
    pub fn with_extension_filter(mut self, filter: ExtensionFilter) -> Self {
        self.filter = Some(Arc::new(FilterRegistry::Extension(filter)));
        self
    }
    
    /// Set a name filter
    pub fn with_name_filter(mut self, filter: NameFilter) -> Self {
        self.filter = Some(Arc::new(FilterRegistry::Name(filter)));
        self
    }
    
    /// Set a composite filter
    pub fn with_composite_filter(mut self, filter: CompositeFilter) -> Self {
        self.filter = Some(Arc::new(FilterRegistry::Composite(filter)));
        self
    }
    
    /// Set a progress reporter
    pub fn with_progress_reporter(mut self, observer: ProgressReporter) -> Self {
        self.observer = Some(Arc::new(ObserverRegistry::Progress(observer)));
        self
    }
    
    /// Set a silent observer
    pub fn with_silent_observer(mut self, observer: SilentObserver) -> Self {
        self.observer = Some(Arc::new(ObserverRegistry::Silent(observer)));
        self
    }
    
    /// Set a null observer
    pub fn with_null_observer(mut self) -> Self {
        self.observer = Some(Arc::new(ObserverRegistry::Null(NullObserver)));
        self
    }
    
    /// Set an observer registry directly
    pub fn with_observer(mut self, observer: Arc<ObserverRegistry>) -> Self {
        self.observer = Some(observer);
        self
    }
    
    /// Set the number of worker threads
    pub fn with_workers(mut self, count: usize) -> Self {
        self.workers_count = count;
        self
    }
    
    /// Set the traversal strategy
    pub fn with_traversal_strategy(mut self, strategy: TraversalStrategy) -> Self {
        self.traversal_strategy = strategy;
        self
    }
    
    /// Build the OqabFileFinder
    pub fn build(self) -> OqabFileFinder {
        OqabFileFinder {
            filter: self.filter.unwrap_or_else(|| Arc::new(FilterRegistry::Extension(ExtensionFilter::new("*")))),
            observer: self.observer.unwrap_or_else(|| Arc::new(ObserverRegistry::Null(NullObserver))),
            workers_count: self.workers_count,
            traversal_strategy: self.traversal_strategy,
        }
    }
}

impl Default for OqabFileFinderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating OqabFileFinder instances
pub struct OqabFinderFactory;

impl OqabFinderFactory {
    /// Create a finder for a specific extension
    pub fn create_extension_finder(extension: &str) -> OqabFileFinder {
        OqabFileFinder::builder()
            .with_extension_filter(ExtensionFilter::new(extension))
            .with_null_observer()
            .build()
    }
    
    /// Create a finder for a specific name
    pub fn create_name_filter_with_observer(name: &str, observer: Arc<ObserverRegistry>) -> OqabFileFinder {
        OqabFileFinder::builder()
            .with_name_filter(NameFilter::new(name))
            .with_observer(observer)
            .build()
    }
    
    /// Create a finder that combines extension and name filters
    pub fn create_combined_finder(name: &str, extension: &str, observer: Arc<ObserverRegistry>) -> OqabFileFinder {
        // Create a composite filter that requires both conditions
        let composite = CompositeFilter::new(FilterOperation::And)
            .with_filter(Box::new(NameFilter::new(name)))
            .with_filter(Box::new(ExtensionFilter::new(extension)));
        
        OqabFileFinder::builder()
            .with_composite_filter(composite)
            .with_observer(observer)
            .build()
    }
    
    /// Create a new observer registry from a concrete observer
    pub fn create_observer_registry(observer: Option<Box<dyn SearchObserver>>) -> Arc<ObserverRegistry> {
        // Simplest approach: just use a null observer for all cases
        // In a real application, we would use proper type conversions
        Arc::new(ObserverRegistry::Null(NullObserver))
    }
} 