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
        if path.is_dir() {
            return FilterResult::Accept;
        }

        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str == self.name || self.name == "*" {
                    FilterResult::Accept
                } else {
                    FilterResult::Reject
                }
            } else {
                FilterResult::Reject
            }
        } else {
            FilterResult::Reject
        }
    }
} 