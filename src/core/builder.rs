use std::sync::Arc;

use crate::{
    core::{
        finder::{FinderConfig, FileFinder},
        registry::{FilterRegistry, ObserverRegistry},
        traversal::{DefaultTraversalStrategy, TraversalStrategy},
    },
    filters::Filter,
};

/// Builder for FileFinder
pub struct FileFinderBuilder {
    config: FinderConfig,
    traversal_strategy: Arc<dyn TraversalStrategy + 'static>,
    filter_registry: Arc<FilterRegistry>,
    observer_registry: Arc<ObserverRegistry>,
}

impl FileFinderBuilder {
    /// Create a new FileFinderBuilder with default settings
    pub fn new() -> Self {
        FileFinderBuilder {
            config: FinderConfig::default(),
            traversal_strategy: Arc::new(DefaultTraversalStrategy::new(true)),
            filter_registry: Arc::new(FilterRegistry::new()),
            observer_registry: Arc::new(ObserverRegistry::new()),
        }
    }

    /// Set the finder configuration
    pub fn with_config(mut self, config: FinderConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the traversal strategy with a Box<dyn TraversalStrategy>
    pub fn with_traversal_strategy(mut self, strategy: Box<dyn TraversalStrategy + 'static>) -> Self {
        self.traversal_strategy = Arc::from(strategy);
        self
    }

    /// Set the number of threads to use for search
    pub fn with_threads(mut self, num_threads: usize) -> Self {
        self.config.num_threads = num_threads;
        self
    }

    /// Set whether to follow symbolic links
    pub fn with_follow_links(mut self, follow_links: bool) -> Self {
        self.config.follow_links = follow_links;
        self
    }

    /// Set the maximum search depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.config.max_depth = Some(max_depth);
        self
    }

    /// Add a filter to the filter registry
    pub fn with_filter<F: Filter + 'static>(mut self, name: &str, filter: F) -> Self {
        {
            let filter_registry = Arc::get_mut(&mut self.filter_registry)
                .expect("Cannot modify filter registry: already shared");
            filter_registry.register(name, filter);
        }
        self
    }

    /// Set the observer registry
    pub fn with_observer_registry(mut self, registry: ObserverRegistry) -> Self {
        self.observer_registry = Arc::new(registry);
        self
    }

    /// Build the FileFinder
    pub fn build(self) -> FileFinder {
        FileFinder::new(
            self.config,
            self.traversal_strategy,
            self.filter_registry,
            self.observer_registry,
        )
    }
}

impl Default for FileFinderBuilder {
    fn default() -> Self {
        Self::new()
    }
} 