pub mod advanced;
pub mod finder;
pub mod composite;

// Re-export key types for easier access
pub use advanced::{SearchObserver, TraversalStrategy};
pub use finder::{FinderFactory, FileFilter};
pub use composite::CompositeFilter; 