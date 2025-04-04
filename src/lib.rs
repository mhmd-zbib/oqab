pub mod commands;
pub mod core;
pub mod cli;
pub mod filters;
pub mod utils;

// Re-export main types
pub use commands::{Command, SearchCommand, HelpCommand};
pub use core::{
    AppConfig,
    FileSearchConfig,
    FileFinder,
    FileFinderBuilder,
    FinderFactory,
    SearchObserver,
    NullObserver,
    ProgressReporter,
    SilentObserver,
    TraversalMode,
    TraversalStrategy,
};
pub use filters::{Filter, FilterResult}; 