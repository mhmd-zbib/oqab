use std::{
    path::{Path, PathBuf},
    sync::Arc,
    io,
};
use log::{debug, error, warn};
use anyhow::{Context, Result};
use crate::{
    core::{
        registry::{FilterRegistry, ObserverRegistry},
        traversal::TraversalStrategy,
        worker::WorkerPool,
        observer::TrackingObserver,
    },
    filters::FilterResult,
};
#[derive(Debug, thiserror::Error)]
pub enum FinderError {
    #[error("Directory access error: {0}")]
    DirectoryAccess(#[from] io::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Worker pool error: {0}")]
    WorkerPool(String),
}
#[derive(Debug, Clone)]
pub struct FinderConfig {
    pub num_threads: usize,
    pub follow_links: bool,
    pub max_depth: Option<usize>,
}
impl Default for FinderConfig {
    fn default() -> Self {
        FinderConfig {
            num_threads: num_cpus::get(),
            follow_links: false,
            max_depth: None,
        }
    }
}
pub struct FileFinder {
    config: FinderConfig,
    traversal_strategy: Arc<dyn TraversalStrategy>,
    filter_registry: Arc<FilterRegistry>,
    observer_registry: Arc<ObserverRegistry>,
}
impl FileFinder {
    pub fn new(
        config: FinderConfig,
        traversal_strategy: Arc<dyn TraversalStrategy>,
        filter_registry: Arc<FilterRegistry>,
        observer_registry: Arc<ObserverRegistry>,
    ) -> Self {
        FileFinder {
            config,
            traversal_strategy,
            filter_registry,
            observer_registry,
        }
    }
    pub fn find(&self, root_dir: &Path) -> Result<Vec<PathBuf>> {
    // Main file search method
        let traversal = Arc::clone(&self.traversal_strategy);
        let filters = Arc::clone(&self.filter_registry);
        let observers = Arc::clone(&self.observer_registry);
        if !root_dir.exists() {
            return Err(FinderError::InvalidPath(format!(
                "Root directory does not exist: {}",
                root_dir.display()
            )).into());
        }
        if !root_dir.is_dir() {
            return Err(FinderError::InvalidPath(format!(
                "Path is not a directory: {}",
                root_dir.display()
            )).into());
        }
        debug!("Searching in {}", root_dir.display());
        if self.config.num_threads <= 1 {
            debug!("Using single-threaded mode");
            let mut current_depth = Vec::new();
            if let Err(e) = process_directory(
                root_dir,
                &traversal,
                &filters,
                &observers,
                &self.config,
                &mut current_depth,
            ) {
                warn!("Error processing directory: {}", e);
            }
        } else {
            debug!("Using {} worker threads", self.config.num_threads);
            let worker_pool = WorkerPool::new(
                self.config.num_threads,
                {
                    let traversal = Arc::clone(&traversal);
                    let filters = Arc::clone(&filters);
                    let observers = Arc::clone(&observers);
                    let config = self.config.clone();
                    move |dir_path| {
                        let mut current_depth = Vec::new();
                        if let Err(e) = process_directory(
                            &dir_path,
                            &traversal,
                            &filters,
                            &observers,
                            &config,
                            &mut current_depth,
                        ) {
                            error!("Failed to process {}: {}", dir_path.display(), e);
                        }
                    }
                },
                {
                    let filters = Arc::clone(&filters);
                    let observers = Arc::clone(&observers);
                    move |file_path| {
                        if filters.apply_all(&file_path) == FilterResult::Accept {
                            observers.notify_file_found(&file_path);
                        }
                    }
                },
            );
            if !worker_pool.submit_directory(root_dir) {
                warn!("Failed to submit directory to worker pool");
            }
            worker_pool.complete();
            worker_pool.join();
        }
        if let Some(tracking_observer) = Self::find_tracking_observer(&observers) {
            match tracking_observer.lock_found_files() {
                Ok(files_guard) => {
                    let mut result = Vec::with_capacity(files_guard.len());
                    for path in files_guard.iter() {
                        result.push(path.clone());
                    }
                    debug!("Found {} matching files", result.len());
                    Ok(result)
                },
                Err(e) => {
                    warn!("Failed to lock found files: {}", e);
                    #[allow(deprecated)]
                    let files = tracking_observer.get_found_files();
                    debug!("Using fallback method - found {} files", files.len());
                    Ok(files)
                }
            }
        } else {
            debug!("No tracking observer found, using direct collection");
            let mut results = Vec::new();
            if let Err(e) = Self::collect_files_direct(
                root_dir,
                &*traversal,
                &filters,
                &mut results,
                self.config.max_depth.unwrap_or(usize::MAX),
                0
            ) {
                warn!("Direct collection error: {}", e);
            }
            debug!("Found {} matching files", results.len());
            Ok(results)
        }
    }
    fn find_tracking_observer(observer_registry: &ObserverRegistry) -> Option<Arc<TrackingObserver>> {
        observer_registry.get_observer_of_type::<TrackingObserver>()
    }
    fn collect_files_direct(
        dir: &Path,
        traversal: &dyn TraversalStrategy,
        filters: &FilterRegistry,
        results: &mut Vec<PathBuf>,
        max_depth: usize,
        current_depth: usize,
    ) -> Result<()> {
        if current_depth >= max_depth || !traversal.should_process_directory(dir) {
            return Ok(());
        }
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?;
        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    warn!("Failed to read directory entry: {}", e);
                    continue;
                }
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    warn!("Failed to determine file type for {}: {}", path.display(), e);
                    continue;
                }
            };
            if file_type.is_dir() {
                if let Err(e) = Self::collect_files_direct(
                    &path,
                    traversal,
                    filters,
                    results,
                    max_depth,
                    current_depth + 1,
                ) {
                    warn!("Error collecting files in subdirectory {}: {}", path.display(), e);
                }
            } else if file_type.is_file() && traversal.should_process_file(&path) && filters.apply_all(&path) == FilterResult::Accept {
                results.push(path);
            }
        }
        Ok(())
    }
    pub fn get_tracking_observer(&self) -> Option<Arc<TrackingObserver>> {
        Self::find_tracking_observer(&self.observer_registry)
    }
}
// Process directory for file search
fn process_directory(
    dir_path: &Path,
    traversal_strategy: &Arc<dyn TraversalStrategy>,
    filter_registry: &Arc<FilterRegistry>,
    observer_registry: &Arc<ObserverRegistry>,
    config: &FinderConfig,
    current_depth: &mut Vec<String>,
) -> Result<()> {
    if let Some(max_depth) = config.max_depth {
        if current_depth.len() >= max_depth {
            return Ok(());
        }
    }
    if !traversal_strategy.should_process_directory(dir_path) {
        return Ok(());
    }
    observer_registry.notify_directory_processed(dir_path);
    let entries = std::fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory entries for: {}", dir_path.display()))?;
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                warn!("Failed to determine file type for {}: {}", path.display(), e);
                continue;
            }
        };
        if file_type.is_dir() {
            if file_type.is_symlink() && !config.follow_links {
                debug!("Skipping symbolic link to directory: {}", path.display());
                continue;
            }
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                current_depth.push(dir_name.to_string());
                if let Err(e) = process_directory(
                    &path,
                    traversal_strategy,
                    filter_registry,
                    observer_registry,
                    config,
                    current_depth
                ) {
                    warn!("Error processing subdirectory {}: {}", path.display(), e);
                }
                current_depth.pop();
            }
        } else if file_type.is_file() && traversal_strategy.should_process_file(&path) {
            if filter_registry.apply_all(&path) == FilterResult::Accept {
                observer_registry.notify_file_found(&path);
            }
        } else if file_type.is_symlink() && config.follow_links {
            match std::fs::read_link(&path) {
                Ok(target) => {
                    let target_path = if target.is_absolute() {
                        target
                    } else {
                        let parent = path.parent().unwrap_or(Path::new(""));
                        parent.join(&target)
                    };
                    match std::fs::metadata(&target_path) {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                                    current_depth.push(dir_name.to_string());
                                    if let Err(e) = process_directory(
                                        &target_path,
                                        traversal_strategy,
                                        filter_registry,
                                        observer_registry,
                                        config,
                                        current_depth
                                    ) {
                                        warn!("Error processing symlinked directory {}: {}",
                                              target_path.display(), e);
                                    }
                                    current_depth.pop();
                                }
                            } else if metadata.is_file() && traversal_strategy.should_process_file(&target_path) {
                                if filter_registry.apply_all(&target_path) == FilterResult::Accept {
                                    observer_registry.notify_file_found(&target_path);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get metadata for symlink target {}: {}",
                                  target_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read symlink {}: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}


