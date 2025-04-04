use std::path::{Path, PathBuf};
use anyhow::Result;
use thiserror::Error;
use walkdir::WalkDir;
use log::{debug, warn, error};
use crate::SearchObserver;

/// Errors specific to file finding operations
#[derive(Error, Debug)]
pub enum FinderError {
    #[error("Failed to access path: {0}")]
    AccessError(String),
    
    #[error("Invalid search path: {0}")]
    InvalidPath(String),
    
    #[error("Traversal error: {0}")]
    TraversalError(String),
    
    #[error("Operation error: {0}")]
    OperationError(String),
}

/// Trait for file filtering logic
pub trait FileFilter: Send + Sync {
    /// Check if a file matches the filter criteria
    fn matches(&self, file_path: &Path) -> bool;
    
    /// Get a description of this filter
    fn description(&self) -> String {
        "Generic file filter".to_string()
    }
}

/// Basic file finder that traverses directories and applies filters
pub struct FileFinder {
    filter: Box<dyn FileFilter>,
}

impl FileFinder {
    /// Create a new file finder with the provided filter
    pub fn new(filter: Box<dyn FileFilter>) -> Self {
        Self { filter }
    }
    
    /// Find files matching the filter criteria in the given path
    pub fn find(&self, search_path: &Path) -> Result<Vec<PathBuf>> {
        let mut results = Vec::new();
        
        // Validate the search path
        if !search_path.exists() {
            return Err(FinderError::InvalidPath(search_path.display().to_string()).into());
        }
        
        debug!("Searching in: {} with filter: {}", 
            search_path.display(), 
            self.filter.description());
        
        // Perform the walk
        let walker = WalkDir::new(search_path)
            .follow_links(true)
            .into_iter();
        
        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    // Skip directories
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        
                        // Apply filter
                        if self.filter.matches(path) {
                            results.push(path.to_path_buf());
                        }
                    }
                },
                Err(err) => {
                    // Log error but continue searching
                    warn!("Error during directory traversal: {}", err);
                    
                    // Check if this is a permission error - match on error type
                    if err.to_string().contains("permission denied") || 
                       err.to_string().contains("access denied") {
                        return Err(FinderError::AccessError(err.to_string()).into());
                    }
                }
            }
        }
        
        Ok(results)
    }
}

/// Filter for file extensions
#[derive(Clone)]
pub struct ExtensionFilter {
    extension: String,
}

impl ExtensionFilter {
    /// Create a new extension filter
    pub fn new(extension: &str) -> Self {
        // Ensure extension starts with a dot
        let ext = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{}", extension)
        };
        
        Self { extension: ext }
    }
}

impl FileFilter for ExtensionFilter {
    fn matches(&self, file_path: &Path) -> bool {
        if let Some(ext) = file_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return format!(".{}", ext_str) == self.extension;
            }
        }
        false
    }
    
    fn description(&self) -> String {
        format!("Extension filter: {}", self.extension)
    }
}

/// Filter for file names
#[derive(Clone)]
pub struct NameFilter {
    name_pattern: String,
}

impl NameFilter {
    /// Create a new name filter
    pub fn new(name_pattern: &str) -> Self {
        Self { name_pattern: name_pattern.to_string() }
    }
}

impl FileFilter for NameFilter {
    fn matches(&self, file_path: &Path) -> bool {
        if let Some(file_name) = file_path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                return name_str.contains(&self.name_pattern);
            }
        }
        false
    }
    
    fn description(&self) -> String {
        format!("Name filter: {}", self.name_pattern)
    }
}

/// Factory for creating file finders with different filters
pub struct FinderFactory;

impl FinderFactory {
    /// Create a finder that filters by file extension
    pub fn create_extension_finder(extension: &str) -> FileFinder {
        FileFinder::new(Box::new(ExtensionFilter::new(extension)))
    }
    
    /// Create a finder that filters by file name
    pub fn create_name_finder(name_pattern: &str) -> FileFinder {
        FileFinder::new(Box::new(NameFilter::new(name_pattern)))
    }
    
    /// Create a finder that filters by both name and extension
    pub fn create_combined_finder(name_pattern: &str, extension: &str) -> FileFinder {
        use crate::search::composite::{CompositeFilter, FilterOperation};
        
        let name_filter = Box::new(NameFilter::new(name_pattern));
        let ext_filter = Box::new(ExtensionFilter::new(extension));
        
        let composite = CompositeFilter::new_with_filters(name_filter, ext_filter, FilterOperation::And);
        
        FileFinder::new(Box::new(composite))
    }
    
    /// Create a finder with an observer, applying the appropriate filter type
    pub fn create_finder_with_observer(
        name_pattern: Option<&str>, 
        extension: Option<&str>, 
        observer: Box<dyn SearchObserver>
    ) -> crate::search::advanced::OqabFileFinder {
        use crate::search::advanced::OqabFinderFactory;
        
        let registry = OqabFinderFactory::create_observer_registry(Some(observer));
        
        match (name_pattern, extension) {
            (Some(name), Some(ext)) => {
                // Both name and extension specified
                OqabFinderFactory::create_combined_finder(name, ext, registry)
            },
            (Some(name), None) => {
                // Name only
                OqabFinderFactory::create_name_filter_with_observer(name, registry)
            },
            (None, Some(ext)) => {
                // Extension only
                let finder = crate::search::advanced::OqabFileFinder::builder()
                    .with_extension_filter(ExtensionFilter::new(ext))
                    .with_observer(registry)
                    .build();
                finder
            },
            (None, None) => {
                // Neither specified - this should not happen
                panic!("No search criteria provided to create_finder_with_observer")
            }
        }
    }
} 