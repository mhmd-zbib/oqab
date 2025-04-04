use std::path::Path;
use crate::filters::{Filter, FilterResult};

/// Filter based on file extension
#[derive(Debug, Clone)]
pub struct ExtensionFilter {
    extension: String,
}

impl ExtensionFilter {
    /// Create a new ExtensionFilter
    pub fn new(extension: &str) -> Self {
        // Normalize extension by removing leading dots
        let extension = extension.trim_start_matches('.');
        ExtensionFilter {
            extension: extension.to_string(),
        }
    }
}

impl Filter for ExtensionFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        if path.is_dir() {
            return FilterResult::Accept;
        }

        if let Some(ext) = path.extension() {
            if ext.to_string_lossy() == self.extension || self.extension == "*" {
                FilterResult::Accept
            } else {
                FilterResult::Reject
            }
        } else {
            // Accept files without extension if the filter is looking for files without extension
            if self.extension.is_empty() {
                FilterResult::Accept
            } else {
                FilterResult::Reject
            }
        }
    }
} 