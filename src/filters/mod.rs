use std::path::Path;

/// Result of filtering a path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    /// Path should be accepted
    Accept,
    /// Path should be rejected
    Reject,
    /// Path should be pruned (reject this and all subdirectories)
    Prune,
}

/// Interface for path filtering
pub trait Filter: Send + Sync {
    /// Filter a path
    fn filter(&self, path: &Path) -> FilterResult;
}

/// Operation to apply to combined filters
#[derive(Debug, Clone, Copy)]
pub enum FilterOperation {
    /// All filters must accept (logical AND)
    And,
    /// Any filter must accept (logical OR)
    Or,
}

pub mod name;
pub mod extension;
pub mod regex;
pub mod size;
pub mod composite;
pub mod date;

pub use name::NameFilter;
pub use extension::ExtensionFilter;
pub use regex::RegexFilter;
pub use size::SizeFilter;
pub use composite::{CompositeFilter, TypedCompositeFilter}; 