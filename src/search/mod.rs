pub mod advanced;
pub mod finder;
pub mod composite;
pub mod service;

// Re-export key types for easier access
pub use advanced::{ObserverRegistry, TraversalStrategy};
pub use finder::{FinderFactory, FileFilter};
pub use composite::CompositeFilter;
pub use service::SearchService; 