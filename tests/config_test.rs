use std::path::PathBuf;
use oqab::core::config::{AppConfig, FileSearchConfig};

#[test]
fn test_app_config_defaults() {
    let config = AppConfig {
        root_dir: PathBuf::from("/test/path"),
        extension: None,
        name: None,
        pattern: None,
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
        size: None,
        depth: None,
        threads: None,
        follow_links: None,
        show_progress: None,
    };
    
    // Check defaults
    assert_eq!(config.root_dir, PathBuf::from("/test/path"));
    assert_eq!(config.extension, None);
    assert_eq!(config.name, None);
    assert_eq!(config.pattern, None);
    assert_eq!(config.min_size, None);
    assert_eq!(config.max_size, None);
    assert_eq!(config.newer_than, None);
    assert_eq!(config.older_than, None);
    assert_eq!(config.size, None);
    assert_eq!(config.depth, None);
    assert_eq!(config.threads, None);
    assert_eq!(config.follow_links, None);
    assert_eq!(config.show_progress, None);
}

#[test]
fn test_file_search_config() {
    let config = FileSearchConfig {
        path: Some(String::from("/test/path")),
        file_extension: Some(String::from("txt")),
        file_name: Some(String::from("test")),
        advanced_search: true,
        thread_count: Some(4),
        show_progress: true,
        recursive: true,
        follow_symlinks: false,
        traversal_mode: Default::default(),
        min_size: Some(1000),
        max_size: Some(5000),
        newer_than: Some(String::from("2023-01-01")),
        older_than: Some(String::from("2023-12-31")),
    };
    
    // Check values
    assert_eq!(config.path, Some(String::from("/test/path")));
    assert_eq!(config.file_extension, Some(String::from("txt")));
    assert_eq!(config.file_name, Some(String::from("test")));
    assert!(config.advanced_search);
    assert_eq!(config.thread_count, Some(4));
    assert!(config.show_progress);
    assert!(config.recursive);
    assert!(!config.follow_symlinks);
    assert_eq!(config.min_size, Some(1000));
    assert_eq!(config.max_size, Some(5000));
    assert_eq!(config.newer_than, Some(String::from("2023-01-01")));
    assert_eq!(config.older_than, Some(String::from("2023-12-31")));
}

#[test]
fn test_file_search_config_defaults() {
    let config = FileSearchConfig {
        path: None,
        file_extension: None,
        file_name: None,
        advanced_search: false,
        thread_count: None,
        show_progress: false,
        recursive: false,
        follow_symlinks: false,
        traversal_mode: Default::default(),
        min_size: None,
        max_size: None,
        newer_than: None,
        older_than: None,
    };
    
    // Check defaults
    assert_eq!(config.path, None);
    assert_eq!(config.file_extension, None);
    assert_eq!(config.file_name, None);
    assert!(!config.advanced_search);
    assert_eq!(config.thread_count, None);
    assert!(!config.show_progress);
    assert!(!config.recursive);
    assert!(!config.follow_symlinks);
    assert_eq!(config.min_size, None);
    assert_eq!(config.max_size, None);
    assert_eq!(config.newer_than, None);
    assert_eq!(config.older_than, None);
} 