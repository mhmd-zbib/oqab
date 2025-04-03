use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;
use log::debug;

/// Trait for file filtering logic
pub trait FileFilter: Send + Sync {
    /// Check if a file matches the filter criteria
    fn matches(&self, file_path: &Path) -> bool;
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
        
        debug!("Searching in: {}", search_path.display());
        
        for entry in WalkDir::new(search_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Skip directories
            if entry.file_type().is_file() {
                let path = entry.path();
                
                // Apply filter
                if self.filter.matches(path) {
                    results.push(path.to_path_buf());
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
    pub fn create_name_and_extension_finder(name_pattern: &str, extension: &str) -> FileFinder {
        use crate::search::composite::{CompositeFilter, FilterOperation};
        
        let name_filter = Box::new(NameFilter::new(name_pattern));
        let ext_filter = Box::new(ExtensionFilter::new(extension));
        
        let composite = CompositeFilter::new(name_filter, ext_filter, FilterOperation::And);
        
        FileFinder::new(Box::new(composite))
    }
    
    /// Create a finder with an extension filter and observer (forwards to advanced finder)
    pub fn create_extension_finder_with_observer(extension: &str, observer: Box<dyn crate::search::SearchObserver>) -> crate::search::advanced::OqabFileFinder {
        crate::search::advanced::OqabFinderFactory::create_extension_finder(extension, observer)
    }
    
    /// Create a finder with a name filter and observer (forwards to advanced finder)
    pub fn create_name_finder_with_observer(name: &str, observer: Box<dyn crate::search::SearchObserver>) -> crate::search::advanced::OqabFileFinder {
        crate::search::advanced::OqabFinderFactory::create_name_filter_with_observer(name, observer)
    }
    
    /// Create a finder with both extension and name filters and observer
    pub fn create_extension_and_name_finder(
        extension: &str, 
        name: &str, 
        observer: Option<Box<dyn crate::search::SearchObserver>>
    ) -> crate::search::advanced::OqabFileFinder {
        crate::search::advanced::OqabFinderFactory::create_combined_finder(name, extension, observer)
    }
} 