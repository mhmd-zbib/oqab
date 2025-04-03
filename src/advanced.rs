use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use std::collections::HashSet;

use crossbeam::channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use walkdir::WalkDir;
use num_cpus;

use crate::finder::{FileFilter, ExtensionFilter};

// Observer pattern for notifications during search process
pub trait SearchObserver: Send + Sync {
    fn on_file_found(&self, path: &Path);
    fn on_directory_entered(&self, path: &Path);
    fn on_directory_error(&self, path: &Path, error: &io::Error);
    fn on_search_completed(&self, total_files: usize, elapsed_time_ms: u128);
}

// Null object pattern
pub struct NullObserver;

impl SearchObserver for NullObserver {
    fn on_file_found(&self, _path: &Path) {}
    fn on_directory_entered(&self, _path: &Path) {}
    fn on_directory_error(&self, _path: &Path, _error: &io::Error) {}
    fn on_search_completed(&self, _total_files: usize, _elapsed_time_ms: u128) {}
}

// Strategy pattern for directory traversal
#[derive(Clone, Copy)]
pub enum TraversalStrategy {
    Standard,     // Uses walkdir
    GitAware,     // Placeholder for future git-awareness
    BreadthFirst, // Placeholder for future BFS implementation
}

impl Default for TraversalStrategy {
    fn default() -> Self {
        TraversalStrategy::Standard
    }
}

// Worker pool for processing files
struct WorkerPool {
    sender: Sender<Option<PathBuf>>,
    receiver: Receiver<Option<PathBuf>>,
    results: Arc<DashMap<String, PathBuf>>,
    filter: Arc<Box<dyn FileFilter>>,
    observer: Arc<dyn SearchObserver>,
}

impl WorkerPool {
    fn new(
        concurrency: usize, 
        filter: Box<dyn FileFilter>,
        observer: Arc<dyn SearchObserver>
    ) -> Self {
        let (sender, receiver) = bounded(concurrency * 2);
        
        Self {
            sender,
            receiver,
            results: Arc::new(DashMap::new()),
            filter: Arc::new(filter),
            observer,
        }
    }
    
    fn start_workers(&self, num_workers: usize) -> Vec<thread::JoinHandle<()>> {
        let mut handles = Vec::with_capacity(num_workers);
        
        for _ in 0..num_workers {
            let receiver = self.receiver.clone();
            let results = self.results.clone();
            let filter = self.filter.clone();
            let observer = self.observer.clone();
            
            let handle = thread::spawn(move || {
                while let Ok(Some(path)) = receiver.recv() {
                    if filter.matches(&path) {
                        observer.on_file_found(&path);
                        
                        // Canonicalize path for deduplication
                        if let Ok(canonical) = path.canonicalize() {
                            let key = canonical.to_string_lossy().to_string();
                            results.insert(key, path);
                        } else {
                            let key = path.to_string_lossy().to_string();
                            results.insert(key, path);
                        }
                    }
                }
            });
            
            handles.push(handle);
        }
        
        handles
    }
    
    fn process_directory(&self, root_dir: &Path) -> io::Result<()> {
        let sender = self.sender.clone();
        let observer = self.observer.clone();
        
        // Use walkdir for traversal
        for entry in WalkDir::new(root_dir).follow_links(true) {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        observer.on_directory_entered(path);
                    } else {
                        if let Err(_) = sender.send(Some(path.to_path_buf())) {
                            break;
                        }
                    }
                }
                Err(err) => {
                    let path = err.path().unwrap_or(Path::new("")).to_path_buf();
                    let io_err = io::Error::new(io::ErrorKind::Other, err.to_string());
                    observer.on_directory_error(&path, &io_err);
                }
            }
        }
        
        // Signal workers that no more files are coming
        for _ in 0..num_cpus::get() {
            let _ = sender.send(None);
        }
        
        Ok(())
    }
}

// Enhanced File Finder with parallel processing and caching
pub struct HyperFileFinder {
    filter: Box<dyn FileFilter>,
    workers: usize,
    observer: Arc<dyn SearchObserver>,
    cache: Arc<DashMap<String, Vec<PathBuf>>>,
    traversal_strategy: TraversalStrategy,
}

// Builder pattern for HyperFileFinder
pub struct HyperFileFinderBuilder {
    filter: Option<Box<dyn FileFilter>>,
    workers: Option<usize>,
    observer: Option<Arc<dyn SearchObserver>>,
    traversal_strategy: Option<TraversalStrategy>,
}

impl HyperFileFinderBuilder {
    pub fn new() -> Self {
        Self {
            filter: None,
            workers: None,
            observer: None,
            traversal_strategy: None,
        }
    }
    
    pub fn with_filter(mut self, filter: Box<dyn FileFilter>) -> Self {
        self.filter = Some(filter);
        self
    }
    
    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }
    
    pub fn with_observer(mut self, observer: Arc<dyn SearchObserver>) -> Self {
        self.observer = Some(observer);
        self
    }
    
    pub fn with_traversal_strategy(mut self, strategy: TraversalStrategy) -> Self {
        self.traversal_strategy = Some(strategy);
        self
    }
    
    pub fn build(self) -> HyperFileFinder {
        HyperFileFinder {
            filter: self.filter.unwrap_or_else(|| Box::new(ExtensionFilter::new("*"))),
            workers: self.workers.unwrap_or_else(|| num_cpus::get()),
            observer: self.observer.unwrap_or_else(|| Arc::new(NullObserver)),
            cache: Arc::new(DashMap::new()),
            traversal_strategy: self.traversal_strategy.unwrap_or_default(),
        }
    }
}

impl HyperFileFinder {
    pub fn find(&self, root_dir: &str) -> io::Result<Vec<PathBuf>> {
        // Generate cache key from filter and directory
        let cache_key = format!("{}:{}", self.filter.name(), root_dir);
        
        // Check if result is in cache
        if let Some(cached_results) = self.cache.get(&cache_key) {
            return Ok(cached_results.clone());
        }
        
        let start_time = Instant::now();
        
        // Create worker pool
        let pool = WorkerPool::new(
            self.workers,
            self.filter.clone(),
            self.observer.clone()
        );
        
        // Start worker threads
        let handles = pool.start_workers(self.workers);
        
        // Process directory
        pool.process_directory(Path::new(root_dir))?;
        
        // Wait for all workers to finish
        for handle in handles {
            let _ = handle.join();
        }
        
        // Collect results
        let results: Vec<PathBuf> = pool.results.iter()
            .map(|entry| entry.value().clone())
            .collect();
            
        // Update cache
        self.cache.insert(cache_key, results.clone());
        
        // Notify observer of search completion
        let elapsed = start_time.elapsed().as_millis();
        self.observer.on_search_completed(results.len(), elapsed);
        
        Ok(results)
    }
}

// Factory for HyperFileFinder
pub struct HyperFinderFactory;

impl HyperFinderFactory {
    pub fn create_extension_finder(extension: &str) -> HyperFileFinder {
        let filter = Box::new(ExtensionFilter::new(extension));
        
        HyperFileFinderBuilder::new()
            .with_filter(filter)
            .with_workers(num_cpus::get())
            .with_traversal_strategy(TraversalStrategy::Standard)
            .build()
    }
} 