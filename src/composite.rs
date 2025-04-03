use std::path::Path;
use crate::finder::FileFilter;

// Composite pattern for combining filters
#[derive(Clone)]
pub enum FilterOperation {
    And,
    Or,
}

#[derive(Clone)]
pub struct CompositeFilter {
    filters: Vec<Box<dyn FileFilter>>,
    operation: FilterOperation,
}

impl CompositeFilter {
    pub fn new(operation: FilterOperation) -> Self {
        Self { 
            filters: Vec::new(),
            operation,
        }
    }
    
    pub fn add_filter(&mut self, filter: Box<dyn FileFilter>) {
        self.filters.push(filter);
    }
}

impl FileFilter for CompositeFilter {
    fn matches(&self, path: &Path) -> bool {
        match self.operation {
            FilterOperation::And => self.filters.iter().all(|filter| filter.matches(path)),
            FilterOperation::Or => self.filters.iter().any(|filter| filter.matches(path)),
        }
    }
    
    fn name(&self) -> String {
        let op_name = match self.operation {
            FilterOperation::And => "AND",
            FilterOperation::Or => "OR",
        };
        
        let filters = self.filters.iter()
            .map(|f| f.name())
            .collect::<Vec<_>>()
            .join(", ");
            
        format!("CompositeFilter({}: {})", op_name, filters)
    }
    
    fn clone_box(&self) -> Box<dyn FileFilter> {
        Box::new(self.clone())
    }
} 