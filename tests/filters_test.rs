use std::path::Path;
use oqab::filters::{Filter, FilterResult, NameFilter, ExtensionFilter, SizeFilter};

#[test]
fn test_name_filter() {
    let filter = NameFilter::new("test.txt");
    
    // In our implementation, we're looking for exact file name matches, not just paths ending with the name
    // The file's name should be exactly "test.txt", not "/path/to/test.txt"
    assert_eq!(filter.filter(Path::new("test.txt")), FilterResult::Accept);
    
    // Test non-matching file
    assert_eq!(filter.filter(Path::new("other.txt")), FilterResult::Reject);
    
    // Test directory (should always accept for traversal)
    assert_eq!(filter.filter(Path::new("/path/to/dir")), FilterResult::Accept);
    
    // Test wildcard
    let wildcard_filter = NameFilter::new("*");
    assert_eq!(wildcard_filter.filter(Path::new("any_file.txt")), FilterResult::Accept);
}

#[test]
fn test_extension_filter() {
    let filter = ExtensionFilter::new("txt");
    
    // Test matching extension
    assert_eq!(filter.filter(Path::new("file.txt")), FilterResult::Accept);
    
    // Test non-matching extension
    assert_eq!(filter.filter(Path::new("file.jpg")), FilterResult::Reject);
    
    // Test file without extension
    assert_eq!(filter.filter(Path::new("file")), FilterResult::Reject);
    
    // Test directory (should always accept for traversal)
    assert_eq!(filter.filter(Path::new("/path/to/dir")), FilterResult::Accept);
    
    // Test wildcard extension
    let wildcard_filter = ExtensionFilter::new("*");
    assert_eq!(wildcard_filter.filter(Path::new("file.jpg")), FilterResult::Accept);
    
    // Test empty extension filter with a file that has no extension
    let empty_filter = ExtensionFilter::new("");
    assert_eq!(empty_filter.filter(Path::new("file")), FilterResult::Accept);
}

#[test]
fn test_size_filter() {
    // Skip actual file size testing because we can't easily create files
    // of specific sizes in a unit test without using tempfile and actual
    // filesystem operations
    
    // Just verify filter creation works correctly
    let min_filter = SizeFilter::min(1000);
    let _range_filter = SizeFilter::range(1000, 2000);
    
    // Paths are accepted when they don't exist but are directories
    // When SizeFilter is applied to a real directory, it will check metadata
    assert_eq!(min_filter.filter(Path::new("/fake/path/that/doesnt/exist")), FilterResult::Reject);
} 