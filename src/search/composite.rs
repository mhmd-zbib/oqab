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
    filters: Vec<Box<dyn FileFilter>>,
    operation: FilterOperation,
}

impl CompositeFilter {
    /// Create a new composite filter with an operation
    pub fn new(operation: FilterOperation) -> Self {
        Self {
            filters: Vec::new(),
            operation,
        }
    }
    
    /// Add a filter to the composite
    pub fn with_filter(mut self, filter: Box<dyn FileFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Add two filters at once and return the composite filter
    pub fn new_with_filters(
        filter1: Box<dyn FileFilter>,
        filter2: Box<dyn FileFilter>,
        operation: FilterOperation,
    ) -> Self {
        let mut filters = Vec::new();
        filters.push(filter1);
        filters.push(filter2);
        
        Self {
            filters,
            operation,
        }
    }
}

impl FileFilter for CompositeFilter {
    fn matches(&self, file_path: &Path) -> bool {
        if self.filters.is_empty() {
            return true; // Empty filter matches everything
        }
        
        match self.operation {
            FilterOperation::And => {
                self.filters.iter().all(|filter| filter.matches(file_path))
            }
            FilterOperation::Or => {
                self.filters.iter().any(|filter| filter.matches(file_path))
            }
        }
    }
} 