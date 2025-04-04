use std::path::{Path, PathBuf};
use oqab::core::observer::{SearchObserver, TrackingObserver, SilentObserver};

#[test]
fn test_tracking_observer() {
    let observer = TrackingObserver::new();
    
    // Add some files
    observer.file_found(Path::new("/path/to/file1.txt"));
    observer.file_found(Path::new("/path/to/file2.txt"));
    observer.file_found(Path::new("/path/to/subdir/file3.txt"));
    
    // Add some directories
    observer.directory_processed(Path::new("/path"));
    observer.directory_processed(Path::new("/path/to"));
    observer.directory_processed(Path::new("/path/to/subdir"));
    
    // Check counts
    assert_eq!(observer.files_count(), 3);
    assert_eq!(observer.directories_count(), 3);
    
    // Check found files
    #[allow(deprecated)]
    let found_files = observer.get_found_files();
    assert_eq!(found_files.len(), 3);
    
    // Verify each file is in the list
    let expected_files = [
        PathBuf::from("/path/to/file1.txt"),
        PathBuf::from("/path/to/file2.txt"),
        PathBuf::from("/path/to/subdir/file3.txt"),
    ];
    
    for expected in &expected_files {
        assert!(found_files.contains(expected));
    }
    
    // Test locking mechanism for found files
    if let Ok(locked_files) = observer.lock_found_files() {
        assert_eq!(locked_files.len(), 3);
    } else {
        panic!("Failed to lock found files");
    }
}

#[test]
fn test_silent_observer() {
    let observer = SilentObserver::new();
    
    // Add some files and directories
    observer.file_found(Path::new("/path/to/file1.txt"));
    observer.file_found(Path::new("/path/to/file2.txt"));
    observer.directory_processed(Path::new("/path/to"));
    
    // Check counts - SilentObserver doesn't track, so these should be 0
    assert_eq!(observer.files_count(), 0);
    assert_eq!(observer.directories_count(), 0);
}

#[test]
fn test_observer_trait_object() {
    // Test that we can use an observer through a trait object
    let observer: Box<dyn SearchObserver> = Box::new(TrackingObserver::new());
    
    observer.file_found(Path::new("/path/to/file.txt"));
    observer.directory_processed(Path::new("/path/to"));
    
    assert_eq!(observer.files_count(), 1);
    assert_eq!(observer.directories_count(), 1);
    
    // Downcast and verify
    let tracking_observer = observer.as_any().downcast_ref::<TrackingObserver>().unwrap();
    #[allow(deprecated)]
    let found_files = tracking_observer.get_found_files();
    assert_eq!(found_files.len(), 1);
    assert_eq!(found_files[0], PathBuf::from("/path/to/file.txt"));
}