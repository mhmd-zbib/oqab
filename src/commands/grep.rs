use anyhow::{Result, Context};
use std::time::{Duration, Instant};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use regex::RegexBuilder;
use console::style;
use log::debug;

use crate::commands::Command;
use crate::core::{ConfigManager, FileSearchConfig};
use crate::utils::search_directory;

/// GrepCommand implements text pattern searching within files
/// 
/// This command follows the Single Responsibility Principle by focusing only on
/// searching for text patterns within files that match specified criteria.
pub struct GrepCommand<'a> {
    config: &'a FileSearchConfig,
    start_time: Instant,
    total_files: RefCell<usize>,
    total_dirs: RefCell<usize>,
    matches_found: RefCell<usize>,
}

impl<'a> GrepCommand<'a> {
    pub fn new(config: &'a FileSearchConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            total_files: RefCell::new(0),
            total_dirs: RefCell::new(0),
            matches_found: RefCell::new(0),
        }
    }

    
    fn search_file(&self, path: &Path, regex: &regex::Regex) -> Result<Vec<(usize, String)>> {
        // Try to open the file, silently skip if permission denied
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                // Skip files we don't have permission to access
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Ok(Vec::new());
                }
                // For other errors, return with context
                return Err(e).with_context(|| format!("Failed to open file: {}", path.display()));
            }
        };
        
        let reader = BufReader::new(file);
        let mut matches = Vec::new();
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    // Skip any errors when reading lines
                    // This handles encoding issues, invalid arguments, and other errors
                    debug!("Skipping line in file {} due to error: {}", path.display(), e);
                    continue;
                }
            };
            
            if regex.is_match(&line) {
                matches.push((line_num + 1, line));
                *self.matches_found.borrow_mut() += 1;
            }
        }
        
        Ok(matches)
    }
    
    fn process_files(&self, files: &[PathBuf], config: &FileSearchConfig) -> Result<()> {
        // Create regex pattern from the config
        let pattern = config.pattern.as_deref().unwrap_or("");
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(config.ignore_case)
            .build()
            .with_context(|| format!("Failed to compile regex pattern: {}", pattern))?;
            
        let mut total_matches = 0;
        
        for file_path in files {
            let matches = self.search_file(file_path, &regex)?;
            
            if !matches.is_empty() {
                if config.files_with_matches {
                    // Only print the filename
                    println!("{}", file_path.display());
                    total_matches += matches.len();
                } else {
                    // Print filename header and matches
                    println!("{}", style(file_path.display()).bold().cyan());
                    
                    // Use a reference to avoid moving matches
                    for (line_num, line) in &matches {
                        if config.line_number {
                            println!("{}: {}", style(line_num).green(), line);
                        } else {
                            println!("{}", line);
                        }
                    }
                    
                    println!(); // Empty line between files
                    total_matches += matches.len();
                }
            }
        }
        
        // Print summary if showing progress
        if config.show_progress {
            let elapsed = self.start_time.elapsed();
            println!("\nFound {} matches in {} files", 
                style(total_matches).bold().green(),
                style(files.len()).bold());
            self.display_performance_metrics(total_matches, elapsed);
        }
        
        Ok(())
    }
    
    fn display_performance_metrics(&self, matches_count: usize, elapsed: Duration) {
        let elapsed_secs = elapsed.as_secs_f64();
        let files_per_sec = if elapsed_secs > 0.0 && *self.total_files.borrow() > 0 {
            *self.total_files.borrow() as f64 / elapsed_secs
        } else {
            0.0
        };
        
        println!("\nPerformance:");
        println!("  Time taken: {:.2} seconds", elapsed_secs);
        println!("  Matches found: {}", matches_count);
        println!("  Files searched: {}", *self.total_files.borrow());
        println!("  Directories searched: {}", *self.total_dirs.borrow());
        println!("  Processing rate: {:.2} files/sec", files_per_sec);
    }
}

impl Command for GrepCommand<'_> {
    fn execute(&self) -> Result<()> {
        // Get the latest configuration from the singleton if available
        let config = if ConfigManager::instance().is_initialized() {
            ConfigManager::instance().get_config()
        } else {
            self.config.clone()
        };
        
        // Create observer for file traversal
        let observer = crate::core::observer::create_observer(config.show_progress);
        
        // Find all files that match the file criteria
        let search_path = std::path::PathBuf::from(config.get_path());
        let files = search_directory(
            &search_path,
            &config,
            &*observer
        ).with_context(|| format!("Failed to search directory: {}", search_path.display()))?;
        
        // Update metrics
        *self.total_files.borrow_mut() = observer.files_count();
        *self.total_dirs.borrow_mut() = observer.directories_count();
        
        // Process the files to find text matches
        if let Err(e) = self.process_files(&files, &config) {
            // Only report errors that aren't permission related
            if !e.to_string().contains("permission denied") {
                return Err(e);
            }
        }
        
        Ok(())
    }
}
