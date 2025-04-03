pub mod config;
pub mod observers;
pub mod commands;
pub mod cli;
pub mod search;

// Re-export main types
pub use commands::{Command, AdvancedSearchCommand, StandardSearchCommand, HelpCommand};
pub use config::FileSearchConfig;
pub use observers::{ProgressReporter, SilentObserver};
pub use search::{SearchObserver, TraversalStrategy, FileFilter}; 