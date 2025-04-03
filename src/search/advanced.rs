use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crossbeam::channel::{unbounded, Sender, Receiver};
use dashmap::DashMap;
use num_cpus;
use serde;

use crate::search::finder::{FileFilter, ExtensionFilter, NameFilter};
use crate::search::composite::{CompositeFilter, FilterOperation};

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
    pub fn new(workers_count: usize, filter: Arc<dyn FileFilter>, results: &DashMap<PathBuf, ()>, observer: &Arc<dyn SearchObserver>) -> Self {
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
        filter: Arc<dyn FileFilter>,
        results: DashMap<PathBuf, ()>,
        observer: Arc<dyn SearchObserver>,
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
        filter: &Arc<dyn FileFilter>,
        results: &DashMap<PathBuf, ()>,
        observer: &Arc<dyn SearchObserver>,
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

/// Enhanced file finder with parallel processing and caching
pub struct HyperFileFinder {
    /// Filter to apply to files
    filter: Arc<dyn FileFilter>,
    /// Observer for search events
    observer: Arc<dyn SearchObserver>,
    /// Number of worker threads
    workers_count: usize,
    /// Directory traversal strategy
    traversal_strategy: TraversalStrategy,
}

impl HyperFileFinder {
    /// Create a new builder for HyperFileFinder
    pub fn builder() -> HyperFileFinderBuilder {
        HyperFileFinderBuilder::new()
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

/// Builder for HyperFileFinder
pub struct HyperFileFinderBuilder {
    filter: Option<Arc<dyn FileFilter>>,
    observer: Option<Arc<dyn SearchObserver>>,
    workers_count: usize,
    traversal_strategy: TraversalStrategy,
}

impl HyperFileFinderBuilder {
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
    pub fn with_filter(mut self, filter: Box<dyn FileFilter>) -> Self {
        self.filter = Some(Arc::new(*filter));
        self
    }
    
    /// Set the search observer
    pub fn with_observer(mut self, observer: Box<dyn SearchObserver>) -> Self {
        self.observer = Some(Arc::new(*observer));
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
    
    /// Build the HyperFileFinder
    pub fn build(self) -> HyperFileFinder {
        HyperFileFinder {
            filter: self.filter.unwrap_or_else(|| Arc::new(ExtensionFilter::new("*"))),
            observer: self.observer.unwrap_or_else(|| Arc::new(NullObserver)),
            workers_count: self.workers_count,
            traversal_strategy: self.traversal_strategy,
        }
    }
}

impl Default for HyperFileFinderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating HyperFileFinder instances
pub struct HyperFinderFactory;

impl HyperFinderFactory {
    /// Create a finder for a specific file extension
    pub fn create_extension_finder(extension: &str, observer: Box<dyn SearchObserver>) -> HyperFileFinder {
        HyperFileFinder::builder()
            .with_filter(Box::new(ExtensionFilter::new(extension)))
            .with_observer(observer)
            .build()
    }
    
    /// Create a finder for multiple criteria
    pub fn create_combined_finder(
        name: &str,
        extension: &str,
        observer: Option<Box<dyn SearchObserver>>
    ) -> HyperFileFinder {
        let name_filter = Box::new(NameFilter::new(name));
        let ext_filter = Box::new(ExtensionFilter::new(extension));
        
        let composite = Box::new(CompositeFilter::new(
            name_filter, 
            ext_filter, 
            FilterOperation::And
        ));
        
        let builder = HyperFileFinder::builder().with_filter(composite);
        
        if let Some(obs) = observer {
            builder.with_observer(obs).build()
        } else {
            builder.build()
        }
    }
} 