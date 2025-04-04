use std::path::Path;
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;
use oqab::filters::{Filter, FilterResult, NameFilter, ExtensionFilter, SizeFilter};

mod helpers;

#[test]
fn test_name_filter() {
    // Create test files to check filter against
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test.txt");
    File::create(&file_path).expect("Failed to create test file");
    
    let filter = NameFilter::new("test.txt");
    
    // Test matching file
    assert_eq!(filter.filter(&file_path), FilterResult::Accept);
    
    // Test non-matching file
    let other_file = temp_dir.path().join("other.txt");
    File::create(&other_file).expect("Failed to create test file");
    assert_eq!(filter.filter(&other_file), FilterResult::Reject);
    
    // Test directory (should always accept for traversal)
    assert_eq!(filter.filter(temp_dir.path()), FilterResult::Accept);
    
    // Test wildcard
    let wildcard_filter = NameFilter::new("*");
    assert_eq!(wildcard_filter.filter(&other_file), FilterResult::Accept);
}

#[test]
fn test_extension_filter() {
    // Create test files to check filter against
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let txt_file = temp_dir.path().join("file.txt");
    let jpg_file = temp_dir.path().join("image.jpg");
    let no_ext_file = temp_dir.path().join("file");
    
    File::create(&txt_file).expect("Failed to create txt file");
    File::create(&jpg_file).expect("Failed to create jpg file");
    File::create(&no_ext_file).expect("Failed to create file with no extension");
    
    let filter = ExtensionFilter::new("txt");
    
    // Test matching extension
    assert_eq!(filter.filter(&txt_file), FilterResult::Accept);
    
    // Test non-matching extension
    assert_eq!(filter.filter(&jpg_file), FilterResult::Reject);
    
    // Test file without extension
    assert_eq!(filter.filter(&no_ext_file), FilterResult::Reject);
    
    // Test directory (should always accept for traversal)
    assert_eq!(filter.filter(temp_dir.path()), FilterResult::Accept);
    
    // Test wildcard extension
    let wildcard_filter = ExtensionFilter::new("*");
    assert_eq!(wildcard_filter.filter(&jpg_file), FilterResult::Accept);
    
    // Test empty extension filter with a file that has no extension
    let empty_filter = ExtensionFilter::new("");
    assert_eq!(empty_filter.filter(&no_ext_file), FilterResult::Accept);
}

#[test]
fn test_size_filter() {
    // Create files with different sizes
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // 500 byte file
    let small_file = temp_dir.path().join("small.txt");
    let small_data = vec![b'a'; 500];
    let mut file = File::create(&small_file).expect("Failed to create small file");
    file.write_all(&small_data).expect("Failed to write data");
    
    // 1500 byte file
    let medium_file = temp_dir.path().join("medium.txt");
    let medium_data = vec![b'b'; 1500];
    let mut file = File::create(&medium_file).expect("Failed to create medium file");
    file.write_all(&medium_data).expect("Failed to write data");
    
    // 2500 byte file
    let large_file = temp_dir.path().join("large.txt");
    let large_data = vec![b'c'; 2500];
    let mut file = File::create(&large_file).expect("Failed to create large file");
    file.write_all(&large_data).expect("Failed to write data");
    
    // Min size filter (files must be at least 1000 bytes)
    let min_filter = SizeFilter::min(1000);
    
    // This file is too small
    assert_eq!(min_filter.filter(&small_file), FilterResult::Reject);
    
    // These files are large enough
    assert_eq!(min_filter.filter(&medium_file), FilterResult::Accept);
    assert_eq!(min_filter.filter(&large_file), FilterResult::Accept);
    
    // Max size filter (files must be at most 2000 bytes)
    let max_filter = SizeFilter::max(2000);
    
    // These files are small enough
    assert_eq!(max_filter.filter(&small_file), FilterResult::Accept);
    assert_eq!(max_filter.filter(&medium_file), FilterResult::Accept);
    
    // This file is too large
    assert_eq!(max_filter.filter(&large_file), FilterResult::Reject);
    
    // Range filter (files must be between 1000 and 2000 bytes)
    let range_filter = SizeFilter::range(1000, 2000);
    
    // This file is too small
    assert_eq!(range_filter.filter(&small_file), FilterResult::Reject);
    
    // This file is in the correct range
    assert_eq!(range_filter.filter(&medium_file), FilterResult::Accept);
    
    // This file is too large
    assert_eq!(range_filter.filter(&large_file), FilterResult::Reject);
    
    // Directories are never filtered by size
    assert_eq!(min_filter.filter(temp_dir.path()), FilterResult::Accept);
} 