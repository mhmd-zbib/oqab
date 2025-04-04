use crate::{
    core::{
        builder::FileFinderBuilder,
        config::AppConfig,
        finder::{FinderConfig, FileFinder},
        observer::NullObserver,
        registry::ObserverRegistry,
        traversal::{DefaultTraversalStrategy, RegexTraversalStrategy, TraversalStrategy},
    },
    filters::{ExtensionFilter, NameFilter, RegexFilter, SizeFilter, date::DateFilter},
};

/// Factory for creating pre-configured FileFinder instances
pub struct FinderFactory;

impl FinderFactory {
    /// Create a new finder for standard search
    pub fn create_standard_finder(config: &AppConfig) -> FileFinder {
        let observer_registry = ObserverRegistry::new();
        observer_registry.register(NullObserver);

        let mut builder = FileFinderBuilder::new()
            .with_threads(config.threads.unwrap_or_else(num_cpus::get))
            .with_follow_links(config.follow_links.unwrap_or(false))
            .with_traversal_strategy(Box::new(DefaultTraversalStrategy::new(true)));

        // Add extension filter if specified
        if let Some(ref ext) = config.extension {
            builder = builder.with_filter("extension", ExtensionFilter::new(ext));
        }

        // Add name filter if specified
        if let Some(ref name) = config.name {
            builder = builder.with_filter("name", NameFilter::new(name));
        }

        // Add regex pattern filter if specified
        if let Some(ref pattern) = config.pattern {
            if let Ok(filter) = RegexFilter::new(pattern) {
                builder = builder.with_filter("pattern", filter);
            }
        }

        // Add size filter if specified
        if let Some(size) = config.size {
            builder = builder.with_filter("size", SizeFilter::min(size));
        } else {
            // Add min size filter if specified
            if let Some(min_size) = config.min_size {
                builder = builder.with_filter("min_size", SizeFilter::min(min_size));
            }
            
            // Add max size filter if specified
            if let Some(max_size) = config.max_size {
                builder = builder.with_filter("max_size", SizeFilter::max(max_size));
            }
        }
        
        // Add date filters if specified
        if let Some(ref newer_than) = config.newer_than {
            if let Ok(filter) = DateFilter::newer_than(newer_than) {
                builder = builder.with_filter("newer_than", filter);
            }
        }
        
        if let Some(ref older_than) = config.older_than {
            if let Ok(filter) = DateFilter::older_than(older_than) {
                builder = builder.with_filter("older_than", filter);
            }
        }

        // Set maximum depth if specified
        if let Some(depth) = config.depth {
            builder = builder.with_max_depth(depth);
        }

        builder.build()
    }

    /// Create a new finder for advanced search with regex patterns
    pub fn create_regex_finder(
        config: &AppConfig,
        include_pattern: Option<&str>,
        exclude_pattern: Option<&str>,
    ) -> Result<FileFinder, regex::Error> {
        let traversal_strategy: Box<dyn TraversalStrategy + 'static> = if include_pattern.is_some() || exclude_pattern.is_some() {
            Box::new(RegexTraversalStrategy::new(include_pattern, exclude_pattern)?)
        } else {
            Box::new(DefaultTraversalStrategy::new(true))
        };

        let observer_registry = ObserverRegistry::new();
        observer_registry.register(NullObserver);

        let mut builder = FileFinderBuilder::new()
            .with_threads(config.threads.unwrap_or_else(num_cpus::get))
            .with_follow_links(config.follow_links.unwrap_or(false))
            .with_traversal_strategy(traversal_strategy);

        // Add extension filter if specified
        if let Some(ref ext) = config.extension {
            builder = builder.with_filter("extension", ExtensionFilter::new(ext));
        }

        // Add name filter if specified
        if let Some(ref name) = config.name {
            builder = builder.with_filter("name", NameFilter::new(name));
        }

        // Add regex pattern filter if specified
        if let Some(ref pattern) = config.pattern {
            if let Ok(filter) = RegexFilter::new(pattern) {
                builder = builder.with_filter("pattern", filter);
            }
        }

        // Add size filter if specified
        if let Some(size) = config.size {
            builder = builder.with_filter("size", SizeFilter::min(size));
        } else {
            // Add min size filter if specified
            if let Some(min_size) = config.min_size {
                builder = builder.with_filter("min_size", SizeFilter::min(min_size));
            }
            
            // Add max size filter if specified
            if let Some(max_size) = config.max_size {
                builder = builder.with_filter("max_size", SizeFilter::max(max_size));
            }
        }
        
        // Add date filters if specified
        if let Some(ref newer_than) = config.newer_than {
            if let Ok(filter) = DateFilter::newer_than(newer_than) {
                builder = builder.with_filter("newer_than", filter);
            }
        }
        
        if let Some(ref older_than) = config.older_than {
            if let Ok(filter) = DateFilter::older_than(older_than) {
                builder = builder.with_filter("older_than", filter);
            }
        }

        // Set maximum depth if specified
        if let Some(depth) = config.depth {
            builder = builder.with_max_depth(depth);
        }

        Ok(builder.build())
    }

    /// Create a custom finder with the specified configuration
    pub fn create_custom_finder(
        config: FinderConfig,
        traversal_strategy: Box<dyn TraversalStrategy + 'static>,
    ) -> FileFinder {
        FileFinderBuilder::new()
            .with_config(config)
            .with_traversal_strategy(traversal_strategy)
            .build()
    }
} 