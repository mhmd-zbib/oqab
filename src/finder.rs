use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

// Strategy pattern - Interface for file filtering
pub trait FileFilter: Send + Sync {
    fn matches(&self, path: &Path) -> bool;
    fn name(&self) -> String;
    fn clone_box(&self) -> Box<dyn FileFilter>;
}

impl Clone for Box<dyn FileFilter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Concrete strategy for extension filtering
#[derive(Clone)]
pub struct ExtensionFilter {
    extension: String,
}

impl ExtensionFilter {
    pub fn new(extension: &str) -> Self {
        // Ensure the extension starts with a dot
        let extension = if !extension.starts_with('.') {
            format!(".{}", extension)
        } else {
            extension.to_string()
        };
        
        Self { extension }
    }
}

impl FileFilter for ExtensionFilter {
    fn matches(&self, path: &Path) -> bool {
        path.extension()
            .map_or(false, |e| format!(".{}", e.to_string_lossy()).eq_ignore_ascii_case(&self.extension))
    }
    
    fn name(&self) -> String {
        format!("ExtensionFilter({})", self.extension)
    }
    
    fn clone_box(&self) -> Box<dyn FileFilter> {
        Box::new(self.clone())
    }
}

// File Finder service (Facade pattern)
pub struct FileFinder {
    filter: Box<dyn FileFilter>,
    parallel_threshold: usize,
}

impl FileFinder {
    pub fn new(filter: Box<dyn FileFilter>) -> Self {
        Self { 
            filter,
            parallel_threshold: 3, // Default threshold
        }
    }
    
    // Builder pattern for customization
    pub fn with_parallel_threshold(mut self, threshold: usize) -> Self {
        self.parallel_threshold = threshold;
        self
    }
    
    pub fn find(&self, root_dir: &str) -> io::Result<Vec<PathBuf>> {
        let matching_files = Arc::new(Mutex::new(Vec::new()));
        self.find_recursive(Path::new(root_dir), matching_files.clone())?;
        
        // Return the collected files
        let result = matching_files.lock().unwrap().clone();
        Ok(result)
    }
    
    fn find_recursive(&self, dir: &Path, matching_files: Arc<Mutex<Vec<PathBuf>>>) -> io::Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        
        // Read directory entries
        let entries: Vec<_> = match fs::read_dir(dir) {
            Ok(entries) => entries.filter_map(Result::ok).collect(),
            Err(e) => {
                eprintln!("Warning: Could not access directory {}: {}", dir.display(), e);
                return Ok(());
            }
        };
        
        // Process directories in parallel
        let subdirs: Vec<_> = entries.iter()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
            .collect();
        
        // Process files in the current directory
        for entry in &entries {
            let path = entry.path();
            if !path.is_dir() && self.filter.matches(&path) {
                let mut files = matching_files.lock().unwrap();
                files.push(path);
            }
        }
        
        // Process subdirectories in parallel if there are more than threshold
        if subdirs.len() > self.parallel_threshold {
            subdirs.par_iter().for_each(|subdir| {
                let _ = self.find_recursive(subdir, Arc::clone(&matching_files));
            });
        } else {
            // Process sequentially for small numbers of directories to avoid overhead
            for subdir in subdirs {
                let _ = self.find_recursive(&subdir, Arc::clone(&matching_files));
            }
        }
        
        Ok(())
    }
}

// Factory pattern for creating finders with specific filters
pub struct FinderFactory;

impl FinderFactory {
    pub fn create_extension_finder(extension: &str) -> FileFinder {
        let filter = Box::new(ExtensionFilter::new(extension));
        FileFinder::new(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_files_with_matching_extension() -> io::Result<()> {
        // Create a temporary directory structure
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        
        // Create test files with different extensions
        create_test_file(temp_path.join("file1.txt"), "test content")?;
        create_test_file(temp_path.join("file2.pdf"), "test content")?;
        create_test_file(temp_path.join("file3.txt"), "test content")?;
        
        // Create a subdirectory with more files
        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir)?;
        create_test_file(subdir.join("file4.txt"), "test content")?;
        create_test_file(subdir.join("file5.pdf"), "test content")?;
        
        // Find .txt files
        let finder = FinderFactory::create_extension_finder(".txt");
        let results = finder.find(temp_path.to_str().unwrap())?;
        
        // We expect 3 .txt files
        assert_eq!(results.len(), 3);
        
        // Clean up is handled automatically by tempdir
        Ok(())
    }
    
    #[test]
    fn test_find_files_with_no_matches() -> io::Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        
        // Create test files
        create_test_file(temp_path.join("file1.txt"), "test content")?;
        create_test_file(temp_path.join("file2.pdf"), "test content")?;
        
        // Search for a non-existent extension
        let finder = FinderFactory::create_extension_finder(".xlsx");
        let results = finder.find(temp_path.to_str().unwrap())?;
        
        // We expect no results
        assert_eq!(results.len(), 0);
        
        Ok(())
    }
    
    fn create_test_file(path: PathBuf, content: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(file, "{}", content)?;
        Ok(())
    }
} 