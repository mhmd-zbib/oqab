use std::path::{Path, PathBuf};
use std::time::Instant;
use log::{debug, warn};
use anyhow::{Context, Result};

use crate::core::{
    config::FileSearchConfig,
    observer::SearchObserver,
};

/// Search statistics for performance tracking
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total time elapsed during search
    pub elapsed_ms: u128,
    /// Number of files found
    pub files_found: usize,
    /// Number of directories processed
    pub dirs_processed: usize,
    /// Number of files processed
    pub files_processed: usize,
}

/// Perform a standard search without worker pool
pub fn search_directory(
    root_dir: &Path, 
    config: &FileSearchConfig,
    observer: &dyn SearchObserver
) -> Result<Vec<PathBuf>> {
    debug!("Beginning search in {}", root_dir.display());
    let start_time = Instant::now();
    
    // Check if the root directory exists
    if !root_dir.exists() {
        return Err(anyhow::anyhow!("Root directory does not exist: {}", root_dir.display()));
    }
    
    if !root_dir.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", root_dir.display()));
    }
    
    // Call the recursive search function
    let mut result = Vec::new();
    if let Err(e) = walk_directory(root_dir, config, observer, &mut result) {
        warn!("Error during directory walk: {}", e);
    }
    
    let elapsed = start_time.elapsed();
    let file_count = observer.files_count();
    let dir_count = observer.directories_count();
    let files_per_sec = if elapsed.as_secs_f32() > 0.0 {
        file_count as f32 / elapsed.as_secs_f32()
    } else {
        0.0
    };
    
    debug!(
        "Search completed in {:.2}s: {} matches, processed {} directories and {} files",
        elapsed.as_secs_f32(),
        result.len(),
        dir_count,
        file_count
    );
    
    debug!("Performance: {:.2} files/sec", files_per_sec);
    
    Ok(result)
}

/// Recursively walk directory to find files
fn walk_directory(
    dir_path: &Path, 
    config: &FileSearchConfig,
    observer: &dyn SearchObserver,
    results: &mut Vec<PathBuf>
) -> Result<()> {
    // Notify observer that we're processing this directory
    observer.directory_processed(dir_path);
    
    // Try to read directory entries
    let entries = match std::fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            // Silently skip directories we don't have permission to access
            // This is common when searching from root directory
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                debug!("Skipping directory due to permission denied: {}", dir_path.display());
                return Ok(());
            }
            // For other errors, return with context
            return Err(e).with_context(|| format!("Failed to read directory entries for: {}", dir_path.display()));
        }
    };
    
    for entry_result in entries {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };
        
        let path = entry.path();
        
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                warn!("Failed to determine file type for {}: {}", path.display(), e);
                continue;
            }
        };
        
        // Process based on file type
        if file_type.is_dir() && config.recursive {
            // Skip symbolic links if not following them
            if file_type.is_symlink() && !config.follow_symlinks {
                debug!("Skipping symbolic link to directory: {}", path.display());
                continue;
            }
            
            // Recursively process subdirectory
            if let Err(e) = walk_directory(&path, config, observer, results) {
                // Only log errors that aren't permission related
                if !e.to_string().contains("permission denied") {
                    warn!("Error processing subdirectory {}: {}", path.display(), e);
                }
            }
        } else if file_type.is_file() {
            let matches = match_file(&path, config);
            
            if matches {
                observer.file_found(&path);
                results.push(path);
            }
        } else if file_type.is_symlink() && config.follow_symlinks {
            // Follow symlinks if enabled
            match std::fs::read_link(&path) {
                Ok(target) => {
                    let target_path = if target.is_absolute() {
                        target
                    } else {
                        // Make path relative to the symlink's directory
                        let parent = path.parent().unwrap_or(Path::new(""));
                        parent.join(&target)
                    };
                    
                    match std::fs::metadata(&target_path) {
                        Ok(metadata) => {
                            if metadata.is_dir() && config.recursive {
                                // Process the directory the symlink points to
                                if let Err(e) = walk_directory(&target_path, config, observer, results) {
                                    warn!("Error processing symlinked directory {}: {}", 
                                          target_path.display(), e);
                                }
                            } else if metadata.is_file() {
                                // Process the file the symlink points to
                                let matches = match_file(&target_path, config);
                                
                                if matches {
                                    observer.file_found(&target_path);
                                    results.push(target_path);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get metadata for symlink target {}: {}", 
                                  target_path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read symlink {}: {}", path.display(), e);
                }
            }
        }
    }
    
    Ok(())
}

/// Check if a file matches the configured criteria
fn match_file(file_path: &Path, config: &FileSearchConfig) -> bool {
    // Check file extension if specified
    if let Some(ref ext) = config.file_extension {
        if let Some(file_ext) = file_path.extension().and_then(|e| e.to_str()) {
            if file_ext.to_lowercase() != ext.to_lowercase() {
                return false;
            }
        } else {
            // File has no extension, but we're looking for one
            return false;
        }
    }
    
    // Check file name if specified
    if let Some(ref name_pattern) = config.file_name {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            // Simple case-insensitive contains check
            if !file_name.to_lowercase().contains(&name_pattern.to_lowercase()) {
                return false;
            }
        } else {
            // File has no name somehow
            return false;
        }
    }
    
    // Check size constraints if specified
    if config.min_size.is_some() || config.max_size.is_some() {
        match std::fs::metadata(file_path) {
            Ok(metadata) => {
                let file_size = metadata.len();
                
                // Check minimum size
                if let Some(min_size) = config.min_size {
                    if file_size < min_size {
                        return false;
                    }
                }
                
                // Check maximum size
                if let Some(max_size) = config.max_size {
                    if file_size > max_size {
                        return false;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get metadata for size check on {}: {}", file_path.display(), e);
                return false;
            }
        }
    }
    
    // Check date constraints if specified
    if config.newer_than.is_some() || config.older_than.is_some() {
        match std::fs::metadata(file_path) {
            Ok(metadata) => {
                // Check newer than constraint
                if let Some(ref newer_than) = config.newer_than {
                    match metadata.modified() {
                        Ok(modified_time) => {
                            let modified_secs = modified_time
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64;
                            
                            if let Ok(newer_time) = newer_than.parse::<i64>() {
                                if modified_secs < newer_time {
                                    return false;
                                }
                            } else {
                                warn!("Invalid newer_than value: {}", newer_than);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get modified time for {}: {}", file_path.display(), e);
                            return false;
                        }
                    }
                }
                
                // Check older than constraint
                if let Some(ref older_than) = config.older_than {
                    match metadata.modified() {
                        Ok(modified_time) => {
                            let modified_secs = modified_time
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64;
                            
                            if let Ok(older_time) = older_than.parse::<i64>() {
                                if modified_secs > older_time {
                                    return false;
                                }
                            } else {
                                warn!("Invalid older_than value: {}", older_than);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get modified time for {}: {}", file_path.display(), e);
                            return false;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to get metadata for date check on {}: {}", file_path.display(), e);
                return false;
            }
        }
    }
    
    // All checks passed
    true
}