use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam::channel::{unbounded, Sender, Receiver};
use dashmap::DashMap;
use num_cpus;
use serde;
use std::any::Any;

use crate::search::finder::{FileFilter, ExtensionFilter, NameFilter};
use crate::search::composite::{CompositeFilter, FilterOperation};
use crate::search::SearchObserver;
use crate::observers::{ProgressReporter, SilentObserver};

/// Directory traversal strategies
#[derive(Copy, Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum TraversalStrategy {
    /// Breadth-first traversal (process directories level by level)
    #[default]
    BreadthFirst,
    /// Depth-first traversal (process directories recursively)
    DepthFirst,
}

/// Observer for search events
pub trait SearchObserver: Send + Sync {
    /// Called when a file matching criteria is found
    fn on_file_found(&self, file: &Path);
    
    /// Called when a directory is entered
    fn on_directory_entry(&self, dir: &Path, entries_count: usize);
    
    /// Called when an error occurs
    fn on_error(&self, error: &io::Error, path: &Path);
    
    /// Called when search is complete
    fn on_search_complete(&self, found_count: usize, dirs_count: usize, duration_ms: u128);
}

/// No-op observer that ignores all events
pub struct NullObserver;

impl SearchObserver for NullObserver {
    fn on_file_found(&self, _file: &Path) {}
    fn on_directory_entry(&self, _dir: &Path, _entries_count: usize) {}
    fn on_error(&self, _error: &io::Error, _path: &Path) {}
    fn on_search_complete(&self, _found_count: usize, _dirs_count: usize, _duration_ms: u128) {}
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
            Err(e) => {
                observer.on_error(&e, dir);
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
                Err(e) => {
                    observer.on_error(&e, dir);
                }
            }
        }
        
        // Notify about directory entry with entry count
        observer.on_directory_entry(dir, entries_vec.len());
        
        for path in entries_vec {
            if path.is_file() && filter.matches(&path) {
                results.insert(path.clone(), ());
                observer.on_file_found(&path);
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

/// Observer registry to avoid Arc<Box<dyn Trait>> issues
enum ObserverRegistry {
    Progress(ProgressReporter),
    Silent(SilentObserver),
    Null(NullObserver),
}

impl SearchObserver for ObserverRegistry {
    fn on_file_found(&self, file: &Path) {
        match self {
            ObserverRegistry::Progress(observer) => observer.on_file_found(file),
            ObserverRegistry::Silent(observer) => observer.on_file_found(file),
            ObserverRegistry::Null(observer) => observer.on_file_found(file),
        }
    }
    
    fn on_directory_entry(&self, dir: &Path, entries_count: usize) {
        match self {
            ObserverRegistry::Progress(observer) => observer.on_directory_entry(dir, entries_count),
            ObserverRegistry::Silent(observer) => observer.on_directory_entry(dir, entries_count),
            ObserverRegistry::Null(observer) => observer.on_directory_entry(dir, entries_count),
        }
    }
    
    fn on_error(&self, error: &io::Error, path: &Path) {
        match self {
            ObserverRegistry::Progress(observer) => observer.on_error(error, path),
            ObserverRegistry::Silent(observer) => observer.on_error(error, path),
            ObserverRegistry::Null(observer) => observer.on_error(error, path),
        }
    }
    
    fn on_search_complete(&self, found_count: usize, dirs_count: usize, duration_ms: u128) {
        match self {
            ObserverRegistry::Progress(observer) => observer.on_search_complete(found_count, dirs_count, duration_ms),
            ObserverRegistry::Silent(observer) => observer.on_search_complete(found_count, dirs_count, duration_ms),
            ObserverRegistry::Null(observer) => observer.on_search_complete(found_count, dirs_count, duration_ms),
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
        
        // Notify observer about search completion
        let duration = start_time.elapsed().as_millis();
        self.observer.on_search_complete(
            found_files.len(),
            dirs_processed.load(Ordering::Relaxed),
            duration,
        );
        
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
                Err(e) => {
                    self.observer.on_error(&e, &dir);
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
            Err(e) => {
                self.observer.on_error(&e, dir);
            }
        }
        
        Ok(())
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
    pub fn create_extension_finder(extension: &str, observer: Box<dyn SearchObserver>) -> OqabFileFinder {
        let builder = OqabFileFinder::builder()
            .with_extension_filter(ExtensionFilter::new(extension));
            
        // Check the concrete type of the observer using downcasting
        if let Some(progress) = check_observer_type::<ProgressReporter>(&*observer) {
            builder.with_progress_reporter(progress).build()
        } else if let Some(silent) = check_observer_type::<SilentObserver>(&*observer) {
            builder.with_silent_observer(silent).build()
        } else {
            builder.with_null_observer().build()
        }
    }
    
    /// Create a finder with a name filter and observer
    pub fn create_name_filter_with_observer(name: &str, observer: Box<dyn SearchObserver>) -> OqabFileFinder {
        let builder = OqabFileFinder::builder()
            .with_name_filter(NameFilter::new(name));
            
        // Check the concrete type of the observer using downcasting
        if let Some(progress) = check_observer_type::<ProgressReporter>(&*observer) {
            builder.with_progress_reporter(progress).build()
        } else if let Some(silent) = check_observer_type::<SilentObserver>(&*observer) {
            builder.with_silent_observer(silent).build()
        } else {
            builder.with_null_observer().build()
        }
    }
    
    /// Create a finder for multiple criteria
    pub fn create_combined_finder(
        name: &str,
        extension: &str,
        observer: Option<Box<dyn SearchObserver>>
    ) -> OqabFileFinder {
        let name_filter = NameFilter::new(name);
        let ext_filter = ExtensionFilter::new(extension);
        
        let composite = CompositeFilter::new(
            Box::new(name_filter.clone()), 
            Box::new(ext_filter), 
            FilterOperation::And
        );
        
        let builder = OqabFileFinder::builder()
            .with_composite_filter(composite);
        
        if let Some(obs) = observer {
            // Check the concrete type of the observer using downcasting
            if let Some(progress) = check_observer_type::<ProgressReporter>(&*obs) {
                builder.with_progress_reporter(progress).build()
            } else if let Some(silent) = check_observer_type::<SilentObserver>(&*obs) {
                builder.with_silent_observer(silent).build()
            } else {
                builder.with_null_observer().build()
            }
        } else {
            builder.with_null_observer().build()
        }
    }
}

/// Helper function to check observer type
fn check_observer_type<T: 'static>(observer: &dyn SearchObserver) -> Option<T> 
where
    T: SearchObserver + Clone
{
    let observer_any = observer as &dyn Any;
    observer_any.downcast_ref::<T>().cloned()
} 