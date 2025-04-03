use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <directory_path> <file_extension>", args[0]);
        eprintln!("Example: {} /path/to/search .pdf", args[0]);
        std::process::exit(1);
    }
    
    let directory_path = &args[1];
    let file_extension = &args[2];
    
    // Ensure the file extension starts with a dot
    let file_extension = if !file_extension.starts_with('.') {
        format!(".{}", file_extension)
    } else {
        file_extension.to_string()
    };
    
    match find_files(directory_path, &file_extension) {
        Ok(files) => {
            if files.is_empty() {
                println!("No files with extension '{}' found in '{}'", file_extension, directory_path);
            } else {
                println!("Found {} file(s) with extension '{}':", files.len(), file_extension);
                for file in files {
                    println!("{}", file.display());
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn find_files(root_dir: &str, extension: &str) -> io::Result<Vec<PathBuf>> {
    let matching_files = Arc::new(Mutex::new(Vec::new()));
    find_files_recursive(Path::new(root_dir), extension, matching_files.clone())?;
    
    // Return the collected files
    let result = matching_files.lock().unwrap().clone();
    Ok(result)
}

fn find_files_recursive(dir: &Path, extension: &str, matching_files: Arc<Mutex<Vec<PathBuf>>>) -> io::Result<()> {
    if dir.is_dir() {
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
            if !path.is_dir() {
                // Check if file has the requested extension
                if path.extension().map_or(false, |e| format!(".{}", e.to_string_lossy()).eq_ignore_ascii_case(extension)) {
                    let mut files = matching_files.lock().unwrap();
                    files.push(path);
                }
            }
        }
        
        // Process subdirectories in parallel if there are more than a few
        if subdirs.len() > 3 {
            subdirs.par_iter().for_each(|subdir| {
                let _ = find_files_recursive(subdir, extension, Arc::clone(&matching_files));
            });
        } else {
            // Process sequentially for small numbers of directories to avoid overhead
            for subdir in subdirs {
                let _ = find_files_recursive(&subdir, extension, Arc::clone(&matching_files));
            }
        }
    }
    
    Ok(())
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
        let results = find_files(temp_path.to_str().unwrap(), ".txt")?;
        
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
        let results = find_files(temp_path.to_str().unwrap(), ".xlsx")?;
        
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
