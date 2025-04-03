use std::path::Path;
use crate::search::FileFilter;

/// Operators for combining filters
#[derive(Debug, Clone, Copy)]
pub enum FilterOperation {
    /// Both filters must match (logical AND)
    And,
    /// Either filter must match (logical OR)
    Or,
}

/// A composite filter that combines two other filters
pub struct CompositeFilter {
    filter1: Box<dyn FileFilter>,
    filter2: Box<dyn FileFilter>,
    operation: FilterOperation,
}

impl CompositeFilter {
    /// Create a new composite filter with two filters and an operation
    pub fn new(
        filter1: Box<dyn FileFilter>,
        filter2: Box<dyn FileFilter>,
        operation: FilterOperation,
    ) -> Self {
        Self {
            filter1,
            filter2,
            operation,
        }
    }
}

impl FileFilter for CompositeFilter {
    fn matches(&self, file_path: &Path) -> bool {
        match self.operation {
            FilterOperation::And => {
                self.filter1.matches(file_path) && self.filter2.matches(file_path)
            }
            FilterOperation::Or => {
                self.filter1.matches(file_path) || self.filter2.matches(file_path)
            }
        }
    }
} 