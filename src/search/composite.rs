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

/// A composite filter that combines multiple filters with boxes
pub struct CompositeFilter {
    filters: Vec<Box<dyn FileFilter>>,
    operation: FilterOperation,
}

/// A type-safe composite filter that combines two specific filter types
pub struct TypedCompositeFilter<F1, F2>
where
    F1: FileFilter,
    F2: FileFilter,
{
    filter1: F1,
    filter2: F2,
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

impl<F1, F2> TypedCompositeFilter<F1, F2>
where
    F1: FileFilter,
    F2: FileFilter,
{
    /// Create a new type-safe composite filter with two existing filters
    pub fn new(filter1: F1, filter2: F2, operation: FilterOperation) -> Self {
        Self {
            filter1,
            filter2,
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
    
    fn description(&self) -> String {
        "Composite filter".to_string()
    }
}

impl<F1, F2> FileFilter for TypedCompositeFilter<F1, F2>
where
    F1: FileFilter,
    F2: FileFilter,
{
    fn matches(&self, file_path: &Path) -> bool {
        match self.operation {
            FilterOperation::And => self.filter1.matches(file_path) && self.filter2.matches(file_path),
            FilterOperation::Or => self.filter1.matches(file_path) || self.filter2.matches(file_path),
        }
    }
    
    fn description(&self) -> String {
        format!(
            "Composite filter: {} {} {}",
            self.filter1.description(),
            match self.operation {
                FilterOperation::And => "AND",
                FilterOperation::Or => "OR",
            },
            self.filter2.description()
        )
    }
} 