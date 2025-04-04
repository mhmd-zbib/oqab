use std::path::Path;
use crate::filters::{Filter, FilterResult};

/// Filter based on file name
#[derive(Debug, Clone)]
pub struct NameFilter {
    name: String,
}

impl NameFilter {
    /// Create a new NameFilter
    pub fn new(name: &str) -> Self {
        NameFilter {
            name: name.to_string(),
        }
    }
}

impl Filter for NameFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        // Always allow directory traversal
        if path.is_dir() {
            return FilterResult::Accept;
        }

        // Get the file name
        match path.file_name() {
            Some(name) => match name.to_str() {
                Some(name_str) if name_str == self.name || self.name == "*" => {
                    FilterResult::Accept
                }
                _ => FilterResult::Reject
            },
            None => FilterResult::Reject
        }
    }
} 