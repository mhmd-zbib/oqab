use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::Lazy;

use crate::core::config::FileSearchConfig;

/// Singleton configuration manager for the application
/// 
/// This implements the Singleton pattern to ensure a single global configuration
/// instance that can be accessed from anywhere in the application.
pub struct ConfigManager {
    config: Arc<Mutex<FileSearchConfig>>,
    initialized: AtomicBool,
}

impl ConfigManager {
    /// Get the singleton instance of the ConfigManager
    pub fn instance() -> &'static ConfigManager {
        static INSTANCE: Lazy<ConfigManager> = Lazy::new(|| {
            ConfigManager {
                config: Arc::new(Mutex::new(FileSearchConfig::new())),
                initialized: AtomicBool::new(false),
            }
        });
        
        &INSTANCE
    }
    
    /// Initialize the configuration with the provided config
    pub fn initialize(&self, config: FileSearchConfig) {
        if !self.initialized.load(Ordering::SeqCst) {
            let mut cfg = self.config.lock().unwrap();
            *cfg = config;
            self.initialized.store(true, Ordering::SeqCst);
        }
    }
    
    /// Get a clone of the current configuration
    pub fn get_config(&self) -> FileSearchConfig {
        let cfg = self.config.lock().unwrap();
        cfg.clone()
    }
    
    /// Update the configuration
    pub fn update_config(&self, config: FileSearchConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = config;
    }
    
    /// Check if the configuration has been initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }
}
