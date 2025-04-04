use anyhow::{Result, Context};
use std::time::{Duration, Instant};
use std::cell::RefCell;
use crate::commands::Command;
use crate::core::{FileSearchConfig, FinderFactory};
use crate::core::observer::{SearchObserver, SilentObserver, TrackingObserver};
use crate::utils::search_directory;

pub struct SearchCommand<'a> {
    config: &'a FileSearchConfig,
    start_time: Instant,
    total_files: RefCell<usize>,
    total_dirs: RefCell<usize>,
}

impl<'a> SearchCommand<'a> {
    pub fn new(config: &'a FileSearchConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            total_files: RefCell::new(0),
            total_dirs: RefCell::new(0),
        }
    }

    fn create_app_config(&self) -> Result<crate::core::config::AppConfig> {
        let app_config = crate::core::config::AppConfig {
            root_dir: match &self.config.path {
                Some(path) => std::path::PathBuf::from(path),
                None => std::env::current_dir()?,
            },
            extension: self.config.file_extension.clone(),
            name: self.config.file_name.clone(),
            pattern: None,
            min_size: self.config.min_size,
            max_size: self.config.max_size,
            newer_than: self.config.newer_than.clone(),
            older_than: self.config.older_than.clone(),
            size: None,
            depth: None,
            threads: self.config.thread_count,
            follow_links: Some(self.config.follow_symlinks),
            show_progress: Some(self.config.show_progress),
        };
        Ok(app_config)
    }
}

impl Command for SearchCommand<'_> {
    fn execute(&self) -> Result<()> {
        let app_config = self.create_app_config()?;

        // Choose observer type based on config
        let observer: Box<dyn SearchObserver> = if self.config.show_progress {
            Box::new(TrackingObserver::new())
        } else {
            Box::new(SilentObserver::new())
        };

        if self.config.advanced_search {
            let finder = FinderFactory::create_standard_finder(&app_config);
            
            let results = finder.find(&app_config.root_dir)
                .with_context(|| format!("Advanced search failed in: {}", app_config.root_dir.display()))?;

            // Get metrics from finder's observer
            if let Some(tracking_observer) = finder.get_tracking_observer() {
                *self.total_files.borrow_mut() = tracking_observer.files_count();
                *self.total_dirs.borrow_mut() = tracking_observer.directories_count();
            }
                
            self.display_results(&results)?;
        } else {
            let search_config = FileSearchConfig {
                path: Some(app_config.root_dir.to_string_lossy().to_string()),
                file_extension: app_config.extension.clone(),
                file_name: app_config.name.clone(),
                advanced_search: false,
                thread_count: app_config.threads,
                show_progress: app_config.show_progress.unwrap_or(true),
                recursive: true,
                follow_symlinks: app_config.follow_links.unwrap_or(false),
                traversal_mode: Default::default(),
                min_size: app_config.min_size,
                max_size: app_config.max_size,
                newer_than: app_config.newer_than.clone(),
                older_than: app_config.older_than.clone(),
            };
            
            let results = search_directory(
                &app_config.root_dir,
                &search_config,
                &*observer
            ).with_context(|| format!("Standard search failed in: {}", app_config.root_dir.display()))?;
            
            // Track search metrics
            *self.total_files.borrow_mut() = observer.files_count();
            *self.total_dirs.borrow_mut() = observer.directories_count();
            
            self.display_results(&results)?;
        }
        
        Ok(())
    }
}

impl SearchCommand<'_> {
    fn display_results(&self, files: &[std::path::PathBuf]) -> Result<()> {
        let elapsed = self.start_time.elapsed();
        
        if !files.is_empty() {
            println!("\nFound {} matching file(s):", files.len());
            for file in files {
                println!("  {}", file.display());
            }
            
            if self.config.show_progress {
                self.display_performance_metrics(files.len(), elapsed);
            }
        } else {
            println!("\nNo matching files found");
            
            if self.config.show_progress {
                self.display_performance_metrics(0, elapsed);
            }
        }
        
        Ok(())
    }
    
    fn display_performance_metrics(&self, files_count: usize, elapsed: Duration) {
        let elapsed_secs = elapsed.as_secs_f64();
        let files_per_sec = if elapsed_secs > 0.0 && files_count > 0 {
            files_count as f64 / elapsed_secs
        } else {
            0.0
        };
        
        println!("\nPerformance:");
        println!("  Time taken: {:.2} seconds", elapsed_secs);
        println!("  Files found: {}", files_count);
        println!("  Files searched: {}", *self.total_files.borrow());
        println!("  Directories searched: {}", *self.total_dirs.borrow());
        println!("  Processing rate: {:.2} files/sec", files_per_sec);
    }
}
