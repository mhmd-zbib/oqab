pub mod commands;
pub mod config;
pub mod observers;
pub mod search;

// Re-export observer types
pub use observers::{ProgressReporter, SilentObserver};
// Re-export search traits and utilities
pub use search::advanced::{SearchObserver, TraversalStrategy, ObserverRegistry, NullObserver};
pub use search::{FileFilter, FinderFactory};
pub mod cli;

// Re-export main types
pub use commands::{Command, AdvancedSearchCommand, StandardSearchCommand, HelpCommand};
pub use config::FileSearchConfig; 