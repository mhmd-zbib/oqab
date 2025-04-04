use std::path::Path;
use crate::filters::{Filter, FilterResult};

/// Filter based on regular expression
#[derive(Debug)]
pub struct RegexFilter {
    regex: regex::Regex,
}

impl RegexFilter {
    /// Create a new RegexFilter
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regex = regex::Regex::new(pattern)?;
        Ok(RegexFilter { regex })
    }
}

impl Filter for RegexFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        if path.is_dir() {
            return FilterResult::Accept;
        }

        let path_str = path.to_string_lossy();
        if self.regex.is_match(&path_str) {
            FilterResult::Accept
        } else {
            FilterResult::Reject
        }
    }
} 