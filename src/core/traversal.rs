use std::{fmt, path::Path};
use serde::{Serialize, Deserialize};

/// Strategy for traversing directories
pub trait TraversalStrategy: Send + Sync {
    /// Check if the given directory should be processed
    fn should_process_directory(&self, path: &Path) -> bool;
    
    /// Check if the given file should be considered
    fn should_process_file(&self, path: &Path) -> bool;
}

/// Default strategy that processes everything except hidden files and directories
#[derive(Debug, Clone)]
pub struct DefaultTraversalStrategy {
    ignore_hidden: bool,
}

impl DefaultTraversalStrategy {
    /// Create a new DefaultTraversalStrategy
    pub fn new(ignore_hidden: bool) -> Self {
        DefaultTraversalStrategy { ignore_hidden }
    }
}

impl TraversalStrategy for DefaultTraversalStrategy {
    fn should_process_directory(&self, path: &Path) -> bool {
        if self.ignore_hidden {
            !is_hidden(path)
        } else {
            true
        }
    }

    fn should_process_file(&self, path: &Path) -> bool {
        if self.ignore_hidden {
            !is_hidden(path)
        } else {
            true
        }
    }
}

/// Directory traversal modes for use in configuration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum TraversalMode {
    /// Breadth-first search
    #[default]
    BreadthFirst,
    /// Depth-first search
    DepthFirst,
}

/// Filter strategy that combines multiple strategies
pub struct CompositeTraversalStrategy {
    strategies: Vec<Box<dyn TraversalStrategy>>,
}

impl fmt::Debug for CompositeTraversalStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeTraversalStrategy")
            .field("strategies_count", &self.strategies.len())
            .finish()
    }
}

impl CompositeTraversalStrategy {
    /// Create a new CompositeTraversalStrategy with the given strategies
    pub fn new(strategies: Vec<Box<dyn TraversalStrategy>>) -> Self {
        CompositeTraversalStrategy { strategies }
    }
}

impl TraversalStrategy for CompositeTraversalStrategy {
    fn should_process_directory(&self, path: &Path) -> bool {
        self.strategies
            .iter()
            .all(|strategy| strategy.should_process_directory(path))
    }

    fn should_process_file(&self, path: &Path) -> bool {
        self.strategies
            .iter()
            .all(|strategy| strategy.should_process_file(path))
    }
}

/// Regex-based traversal strategy
pub struct RegexTraversalStrategy {
    include_pattern: Option<regex::Regex>,
    exclude_pattern: Option<regex::Regex>,
}

impl fmt::Debug for RegexTraversalStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegexTraversalStrategy")
            .field("has_include_pattern", &self.include_pattern.is_some())
            .field("has_exclude_pattern", &self.exclude_pattern.is_some())
            .finish()
    }
}

impl RegexTraversalStrategy {
    /// Create a new RegexTraversalStrategy with the given patterns
    pub fn new(
        include_pattern: Option<&str>,
        exclude_pattern: Option<&str>,
    ) -> Result<Self, regex::Error> {
        let include_regex = match include_pattern {
            Some(pattern) => Some(regex::Regex::new(pattern)?),
            None => None,
        };
        
        let exclude_regex = match exclude_pattern {
            Some(pattern) => Some(regex::Regex::new(pattern)?),
            None => None,
        };
        
        Ok(RegexTraversalStrategy {
            include_pattern: include_regex,
            exclude_pattern: exclude_regex,
        })
    }
}

impl TraversalStrategy for RegexTraversalStrategy {
    fn should_process_directory(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        if let Some(ref exclude) = self.exclude_pattern {
            if exclude.is_match(&path_str) {
                return false;
            }
        }
        
        if let Some(ref include) = self.include_pattern {
            include.is_match(&path_str)
        } else {
            true
        }
    }
    
    fn should_process_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        if let Some(ref exclude) = self.exclude_pattern {
            if exclude.is_match(&path_str) {
                return false;
            }
        }
        
        if let Some(ref include) = self.include_pattern {
            include.is_match(&path_str)
        } else {
            true
        }
    }
}

/// Check if a path is hidden (starts with "." on Unix or has hidden attribute on Windows)
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
        || cfg!(windows) && has_hidden_attribute(path)
}

#[cfg(windows)]
fn has_hidden_attribute(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    
    path.metadata()
        .map(|metadata| (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn has_hidden_attribute(_path: &Path) -> bool {
    false
} 