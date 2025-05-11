pub mod builder;
pub mod config;
pub mod factory;
pub mod finder;
pub mod observer;
pub mod platform;
pub mod registry;
pub mod singleton;
pub mod traversal;
pub mod worker;

// Re-export commonly used types
pub use self::builder::FileFinderBuilder;
pub use self::config::{AppConfig, FileSearchConfig};
pub use self::factory::FinderFactory;
pub use self::finder::FileFinder;
pub use self::observer::{NullObserver, ProgressReporter, SearchObserver, SilentObserver};
pub use self::platform::Platform;
pub use self::registry::{FilterRegistry, ObserverRegistry};
pub use self::singleton::ConfigManager;
pub use self::traversal::{DefaultTraversalStrategy, TraversalMode, TraversalStrategy}; 