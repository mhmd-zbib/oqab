use std::path::Path;
use crate::filters::{Filter, FilterResult};

/// Filter based on file size
#[derive(Debug, Clone)]
pub struct SizeFilter {
    min_size: u64,
}

impl SizeFilter {
    /// Create a new SizeFilter
    pub fn new(min_size: u64) -> Self {
        SizeFilter { min_size }
    }
}

impl Filter for SizeFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        if path.is_dir() {
            return FilterResult::Accept;
        }

        match std::fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() >= self.min_size {
                    FilterResult::Accept
                } else {
                    FilterResult::Reject
                }
            }
            Err(_) => FilterResult::Reject,
        }
    }
} 