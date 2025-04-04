use std::path::Path;
use crate::filters::{Filter, FilterResult};

/// Filter that matches files within a size range
#[derive(Debug)]
pub struct SizeFilter {
    min_size: Option<u64>,
    max_size: Option<u64>,
}

impl SizeFilter {
    /// Create a new size filter with the given size in bytes
    pub fn new(min_size: Option<u64>, max_size: Option<u64>) -> Self {
        Self { min_size, max_size }
    }
    
    /// Create a size filter with just a minimum size
    pub fn min(size: u64) -> Self {
        Self { min_size: Some(size), max_size: None }
    }
    
    /// Create a size filter with just a maximum size
    pub fn max(size: u64) -> Self {
        Self { min_size: None, max_size: Some(size) }
    }
    
    /// Create a size filter with both minimum and maximum size
    pub fn range(min: u64, max: u64) -> Self {
        Self { min_size: Some(min), max_size: Some(max) }
    }
}

impl Filter for SizeFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        // Try to get file metadata
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => return FilterResult::Reject,
        };
        
        // Get file size
        let file_size = metadata.len();
        
        // Check against minimum size if specified
        if let Some(min_size) = self.min_size {
            if file_size < min_size {
                return FilterResult::Reject;
            }
        }
        
        // Check against maximum size if specified
        if let Some(max_size) = self.max_size {
            if file_size > max_size {
                return FilterResult::Reject;
            }
        }
        
        FilterResult::Accept
    }
} 