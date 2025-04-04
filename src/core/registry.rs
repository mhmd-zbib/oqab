use std::{
    any::TypeId,
    collections::HashMap,
    fmt,
    path::Path,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use anyhow::Result;
use log::warn;

use crate::{
    core::observer::{NullObserver, SearchObserver},
    filters::{Filter, FilterResult},
};

/// Registry for filters used in search operations
pub struct FilterRegistry {
    filters: HashMap<String, Box<dyn Filter>>,
}

impl fmt::Debug for FilterRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterRegistry")
            .field("filters_count", &self.filters.len())
            .finish()
    }
}

impl FilterRegistry {
    /// Create a new empty FilterRegistry
    pub fn new() -> Self {
        FilterRegistry {
            filters: HashMap::new(),
        }
    }

    /// Register a filter with the given name
    pub fn register<F>(&mut self, name: &str, filter: F) -> &mut Self
    where
        F: Filter + 'static,
    {
        self.filters.insert(name.to_string(), Box::new(filter));
        self
    }

    /// Get a filter by name
    pub fn get(&self, name: &str) -> Option<&dyn Filter> {
        self.filters.get(name).map(|f| f.as_ref())
    }

    /// Remove a filter by name
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn Filter>> {
        self.filters.remove(name)
    }

    /// Apply all filters to a path
    pub fn apply_all(&self, path: &Path) -> FilterResult {
        for filter in self.filters.values() {
            let result = filter.filter(path);
            if result != FilterResult::Accept {
                return result;
            }
        }
        FilterResult::Accept
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for observers that can be notified of search events
pub struct ObserverRegistry {
    observers: RwLock<Vec<Arc<dyn SearchObserver>>>,
}

impl fmt::Debug for ObserverRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let observers = match self.observers.read() {
            Ok(guard) => guard.len(),
            Err(_) => {
                warn!("Failed to acquire read lock for ObserverRegistry debug");
                0
            }
        };
        
        f.debug_struct("ObserverRegistry")
            .field("observers_count", &observers)
            .finish()
    }
}

impl Clone for ObserverRegistry {
    fn clone(&self) -> Self {
        // Create a new empty registry
        let new_registry = ObserverRegistry::new();
        
        // Copy the observers if we can get the lock
        if let Ok(observers) = self.observers.read() {
            if let Ok(mut new_observers) = new_registry.observers.write() {
                for observer in observers.iter() {
                    new_observers.push(Arc::clone(observer));
                }
            } else {
                warn!("Failed to acquire write lock when cloning ObserverRegistry");
            }
        } else {
            warn!("Failed to acquire read lock when cloning ObserverRegistry");
        }
        
        new_registry
    }
}

impl ObserverRegistry {
    /// Create a new empty ObserverRegistry
    pub fn new() -> Self {
        ObserverRegistry {
            observers: RwLock::new(Vec::new()),
        }
    }

    /// Register an observer
    pub fn register<O>(&self, observer: O) -> &Self
    where
        O: SearchObserver + 'static,
    {
        if let Ok(mut observers) = self.observers.write() {
            observers.push(Arc::new(observer));
        } else {
            warn!("Failed to register observer: could not acquire write lock");
        }
        self
    }

    /// Register an already Arc-wrapped observer
    pub fn register_arc(&self, observer: Arc<dyn SearchObserver>) -> &Self {
        if let Ok(mut observers) = self.observers.write() {
            observers.push(observer);
        } else {
            warn!("Failed to register Arc observer: could not acquire write lock");
        }
        self
    }

    // Helper method to safely acquire read lock
    fn read_observers(&self) -> Result<RwLockReadGuard<'_, Vec<Arc<dyn SearchObserver>>>> {
        self.observers.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock: poisoned lock"))
    }

    /// Notify all observers that a file was found
    pub fn notify_file_found(&self, path: &Path) {
        let observers = match self.read_observers() {
            Ok(obs) => obs,
            Err(e) => {
                warn!("Failed to notify observers of file found: {}", e);
                return;
            }
        };
        
        if observers.is_empty() {
            return;
        }

        for observer in observers.iter() {
            observer.file_found(path);
        }
    }

    /// Notify all observers that a directory was processed
    pub fn notify_directory_processed(&self, path: &Path) {
        let observers = match self.read_observers() {
            Ok(obs) => obs,
            Err(e) => {
                warn!("Failed to notify observers of directory processed: {}", e);
                return;
            }
        };
        
        if observers.is_empty() {
            return;
        }

        for observer in observers.iter() {
            observer.directory_processed(path);
        }
    }

    /// Get total file count from all observers
    pub fn files_count(&self) -> usize {
        let observers = match self.read_observers() {
            Ok(obs) => obs,
            Err(e) => {
                warn!("Failed to get file count: {}", e);
                return 0;
            }
        };
        
        if observers.is_empty() {
            return 0;
        }

        observers.iter().map(|o| o.files_count()).sum()
    }

    /// Get total directory count from all observers
    pub fn directories_count(&self) -> usize {
        let observers = match self.read_observers() {
            Ok(obs) => obs,
            Err(e) => {
                warn!("Failed to get directory count: {}", e);
                return 0;
            }
        };
        
        if observers.is_empty() {
            return 0;
        }

        observers.iter().map(|o| o.directories_count()).sum()
    }

    /// Get an observer of a specific type
    /// 
    /// Returns the first observer that matches the specified type
    pub fn get_observer_of_type<T: 'static>(&self) -> Option<Arc<T>> {
        let observers = match self.read_observers() {
            Ok(obs) => obs,
            Err(e) => {
                warn!("Failed to get observer of type: {}", e);
                return None;
            }
        };
        
        for observer in observers.iter() {
            // Try to downcast the observer reference to the target type
            if let Some(specific_observer) = Self::downcast_observer::<T>(Arc::clone(observer)) {
                return Some(specific_observer);
            }
        }
        
        None
    }
    
    /// Helper method to downcast an observer to a specific type
    /// Uses the TypeId to check if the underlying type matches the target type
    fn downcast_observer<T: 'static>(observer: Arc<dyn SearchObserver>) -> Option<Arc<T>> {
        // Use std::any::TypeId to compare types
        let target_type_id = TypeId::of::<T>();
        
        // Get the type_id of the underlying concrete type by using as_any
        let observer_type_id = observer.as_any().type_id();
        let is_matching_type = observer_type_id == target_type_id;
        
        if is_matching_type {
            // This is where we use Arc::into_raw and from_raw to perform
            // the cast safely, required with Arc
            unsafe {
                // Convert Arc<dyn SearchObserver> to *const dyn SearchObserver
                let ptr = Arc::into_raw(observer);
                
                // Cast to *const T
                let typed_ptr = ptr as *const T;
                
                // Convert back to Arc<T>
                let typed_arc = Arc::from_raw(typed_ptr);
                
                // Create a clone to avoid dropping the original Arc when this function returns
                let result = Arc::clone(&typed_arc);
                
                // Convert typed_arc back to raw and then back to original type
                // to avoid dropping the original allocation
                let _ = Arc::into_raw(typed_arc);
                
                Some(result)
            }
        } else {
            None
        }
    }
}

impl Default for ObserverRegistry {
    fn default() -> Self {
        let registry = Self::new();
        registry.register(NullObserver);
        registry
    }
} 