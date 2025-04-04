use std::{fmt, path::Path};
use crate::filters::{Filter, FilterOperation, FilterResult};

/// A composite filter that combines multiple filters
pub struct CompositeFilter {
    filters: Vec<Box<dyn Filter>>,
    operation: FilterOperation,
}

impl fmt::Debug for CompositeFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeFilter")
            .field("filters_count", &self.filters.len())
            .field("operation", &self.operation)
            .finish()
    }
}

impl CompositeFilter {
    /// Create a new CompositeFilter with the given operation
    pub fn new(operation: FilterOperation) -> Self {
        CompositeFilter {
            filters: Vec::new(),
            operation,
        }
    }

    /// Add a filter to the composite
    pub fn add_filter<F: Filter + 'static>(&mut self, filter: F) -> &mut Self {
        self.filters.push(Box::new(filter));
        self
    }
}

impl Filter for CompositeFilter {
    fn filter(&self, path: &Path) -> FilterResult {
        if self.filters.is_empty() {
            return FilterResult::Accept;
        }

        match self.operation {
            FilterOperation::And => {
                let mut result = FilterResult::Accept;
                
                for filter in &self.filters {
                    match filter.filter(path) {
                        FilterResult::Accept => continue,
                        FilterResult::Reject => {
                            result = FilterResult::Reject;
                            break;
                        }
                        FilterResult::Prune => {
                            result = FilterResult::Prune;
                            break;
                        }
                    }
                }
                
                result
            }
            FilterOperation::Or => {
                // For OR, we need at least one Accept
                let mut found_accept = false;
                let mut found_prune = false;
                
                for filter in &self.filters {
                    match filter.filter(path) {
                        FilterResult::Accept => {
                            found_accept = true;
                            break;
                        }
                        FilterResult::Prune => {
                            found_prune = true;
                        }
                        FilterResult::Reject => continue,
                    }
                }
                
                if found_accept {
                    FilterResult::Accept
                } else if found_prune {
                    FilterResult::Prune
                } else {
                    FilterResult::Reject
                }
            }
        }
    }
}

/// A type-safe composite filter using generics
#[derive(Debug)]
pub struct TypedCompositeFilter<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    filter1: F1,
    filter2: F2,
    operation: FilterOperation,
}

impl<F1, F2> TypedCompositeFilter<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    /// Create a new TypedCompositeFilter with the given filters and operation
    pub fn new(filter1: F1, filter2: F2, operation: FilterOperation) -> Self {
        TypedCompositeFilter {
            filter1,
            filter2,
            operation,
        }
    }
}

impl<F1, F2> Filter for TypedCompositeFilter<F1, F2>
where
    F1: Filter,
    F2: Filter,
{
    fn filter(&self, path: &Path) -> FilterResult {
        match self.operation {
            FilterOperation::And => {
                match self.filter1.filter(path) {
                    FilterResult::Accept => self.filter2.filter(path),
                    other => other,
                }
            }
            FilterOperation::Or => {
                match self.filter1.filter(path) {
                    FilterResult::Accept => FilterResult::Accept,
                    FilterResult::Prune => {
                        match self.filter2.filter(path) {
                            FilterResult::Accept => FilterResult::Accept,
                            _ => FilterResult::Prune,
                        }
                    }
                    FilterResult::Reject => self.filter2.filter(path),
                }
            }
        }
    }
} 