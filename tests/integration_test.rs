use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use oqab::core::config::{AppConfig, FileSearchConfig};
use oqab::core::finder::FileFinder;
use oqab::core::FinderFactory;
use oqab::utils::search_directory;
use oqab::core::observer::TrackingObserver;

// Helper function to create a test directory structure
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create some subdirectories
    let subdir1 = temp_dir.path().join("subdir1");
    let subdir2 = temp_dir.path().join("subdir2");
    let subdir3 = temp_dir.path().join("subdir2/subdir3");
    
    fs::create_dir(&subdir1).expect("Failed to create subdir1");
    fs::create_dir(&subdir2).expect("Failed to create subdir2");
    fs::create_dir(&subdir3).expect("Failed to create subdir3");
    
    // Create some test files with different extensions and sizes
    create_test_file(&temp_dir.path().join("file1.txt"), 1000);
    create_test_file(&temp_dir.path().join("file2.log"), 2000);
    create_test_file(&subdir1.join("file3.txt"), 1500);
    create_test_file(&subdir1.join("image.jpg"), 3000);
    create_test_file(&subdir2.join("document.pdf"), 5000);
    create_test_file(&subdir3.join("config.txt"), 500);
    
    temp_dir
}

// Helper function to create a test file with a specific size
fn create_test_file(path: &Path, size: usize) {
    let mut file = File::create(path).expect("Failed to create test file");
    let data = vec![b'a'; size];
    file.write_all(&data).expect("Failed to write test data");
}

#[test]
fn test_finder_factory_create_standard_finder() {
    let temp_dir = create_test_directory();
    
    let app_config = AppConfig {
        root_dir: temp_dir.path().to_path_buf(),
        extension: Some("txt".to_string()),
        name: None,
        pattern: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        size: None,
        depth: None,
        threads: Some(2),
        follow_links: Some(false),
        show_progress: Some(true),
    };
    
    let finder = FinderFactory::create_standard_finder(&app_config);
    let results = finder.find(temp_dir.path()).expect("Find operation failed");
    
    // We should find 3 .txt files
    assert_eq!(results.len(), 3);
    
    // Verify each path has the correct extension
    for path in &results {
        assert_eq!(path.extension().unwrap(), "txt");
    }
}

#[test]
fn test_search_directory_with_size_filter() {
    let temp_dir = create_test_directory();
    
    let config = FileSearchConfig {
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        file_extension: None,
        file_name: None,
        advanced_search: false,
        thread_count: None,
        show_progress: true,
        recursive: true,
        follow_symlinks: false,
        traversal_mode: Default::default(),
        min_size: Some(2000), // Only files >= 2000 bytes
        max_size: None,
        newer_than: None,
        older_than: None,
    };
    
    let observer = TrackingObserver::new();
    let results = search_directory(
        temp_dir.path(),
        &config,
        &observer
    ).expect("Search operation failed");
    
    // We should find 3 files >= 2000 bytes
    assert_eq!(results.len(), 3);
    
    // Check that each result is at least 2000 bytes
    for path in &results {
        let metadata = fs::metadata(path).expect("Failed to get file metadata");
        assert!(metadata.len() >= 2000);
    }
}

#[test]
fn test_recursive_search() {
    let temp_dir = create_test_directory();
    
    // First, test non-recursive search
    let non_recursive_config = FileSearchConfig {
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        file_extension: None,
        file_name: None,
        advanced_search: false,
        thread_count: None,
        show_progress: false,
        recursive: false, // Non-recursive search
        follow_symlinks: false,
        traversal_mode: Default::default(),
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
    };
    
    let observer1 = TrackingObserver::new();
    let non_recursive_results = search_directory(
        temp_dir.path(),
        &non_recursive_config,
        &observer1
    ).expect("Non-recursive search failed");
    
    // We should only find files in the root directory (2 files)
    assert_eq!(non_recursive_results.len(), 2);
    
    // Now test recursive search
    let recursive_config = FileSearchConfig {
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        file_extension: None,
        file_name: None,
        advanced_search: false,
        thread_count: None,
        show_progress: false,
        recursive: true, // Recursive search
        follow_symlinks: false,
        traversal_mode: Default::default(),
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
    };
    
    let observer2 = TrackingObserver::new();
    let recursive_results = search_directory(
        temp_dir.path(),
        &recursive_config,
        &observer2
    ).expect("Recursive search failed");
    
    // We should find all 6 files
    assert_eq!(recursive_results.len(), 6);
}