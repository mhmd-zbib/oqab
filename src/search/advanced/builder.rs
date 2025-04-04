use std::sync::Arc;

use crate::search::{
    advanced::{
        finder::{FinderConfig, OqabFileFinder},
        registry::{FilterRegistry, ObserverRegistry},
        traversal::{DefaultTraversalStrategy, TraversalStrategy},
    },
    filter::Filter,
};

/// Builder for OqabFileFinder
pub struct OqabFileFinderBuilder {
    config: FinderConfig,
    traversal_strategy: Arc<dyn TraversalStrategy>,
    filter_registry: Arc<FilterRegistry>,
    observer_registry: Arc<ObserverRegistry>,
}

impl OqabFileFinderBuilder {
    /// Create a new OqabFileFinderBuilder with default settings
    pub fn new() -> Self {
        OqabFileFinderBuilder {
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

    /// Set the traversal strategy
    pub fn with_traversal_strategy<T: TraversalStrategy + 'static>(mut self, strategy: T) -> Self {
        self.traversal_strategy = Arc::new(strategy);
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
    pub fn with_filter<F: Filter + 'static>(self, name: &str, filter: F) -> Self {
        {
            let registry = Arc::get_mut(&mut self.filter_registry)
                .expect("Cannot modify filter registry: already shared");
            registry.register(name, filter);
        }
        self
    }

    /// Set the observer registry
    pub fn with_observer_registry(mut self, registry: ObserverRegistry) -> Self {
        self.observer_registry = Arc::new(registry);
        self
    }

    /// Build the OqabFileFinder
    pub fn build(self) -> OqabFileFinder {
        OqabFileFinder::new(
            self.config,
            self.traversal_strategy,
            self.filter_registry,
            self.observer_registry,
        )
    }
}

impl Default for OqabFileFinderBuilder {
    fn default() -> Self {
        Self::new()
    }
} 