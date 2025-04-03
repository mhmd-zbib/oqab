use std::path::Path;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::advanced::{SearchObserver, TraversalStrategy};
use crate::advanced::HyperFinderFactory;

// Command pattern
pub trait Command {
    fn execute(&self) -> i32;
}

// Progress observer for real-time search feedback
pub struct ProgressObserver {
    dirs_scanned: AtomicUsize,
    files_found: AtomicUsize,
    errors: AtomicUsize,
    show_errors: bool,
}

impl ProgressObserver {
    pub fn new(show_errors: bool) -> Self {
        Self {
            dirs_scanned: AtomicUsize::new(0),
            files_found: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            show_errors,
        }
    }
}

impl SearchObserver for ProgressObserver {
    fn on_file_found(&self, _path: &Path) {
        let count = self.files_found.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 100 == 0 {
            println!("Found {} files so far...", count);
        }
    }
    
    fn on_directory_entered(&self, _path: &Path) {
        let count = self.dirs_scanned.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 100 == 0 {
            println!("Scanned {} directories...", count);
        }
    }
    
    fn on_directory_error(&self, path: &Path, error: &io::Error) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        if self.show_errors {
            eprintln!("Error accessing '{}': {}", path.display(), error);
        }
    }
    
    fn on_search_completed(&self, total_files: usize, elapsed_time_ms: u128) {
        println!("\nSearch completed:");
        println!("  Directories scanned: {}", self.dirs_scanned.load(Ordering::Relaxed));
        println!("  Files found: {}", total_files);
        println!("  Errors encountered: {}", self.errors.load(Ordering::Relaxed));
        
        let elapsed_sec = elapsed_time_ms as f64 / 1000.0;
        println!("  Time taken: {:.2} seconds", elapsed_sec);
        
        if elapsed_sec > 0.0 {
            let files_per_sec = total_files as f64 / elapsed_sec;
            println!("  Performance: {:.2} files/second", files_per_sec);
        }
    }
}

// Enhanced version of FindFilesCommand that uses the hyper finder
pub struct HyperFindFilesCommand {
    directory: String,
    extension: String,
    name: Option<String>,
    traversal_strategy: TraversalStrategy,
    show_progress: bool,
    show_errors: bool,
}

impl HyperFindFilesCommand {
    pub fn new(directory: String, extension: String) -> Self {
        Self { 
            directory, 
            extension,
            name: None,
            traversal_strategy: TraversalStrategy::Standard,
            show_progress: true,
            show_errors: false,
        }
    }
    
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn with_traversal_strategy(mut self, strategy: TraversalStrategy) -> Self {
        self.traversal_strategy = strategy;
        self
    }
    
    pub fn with_progress(mut self, show_progress: bool) -> Self {
        self.show_progress = show_progress;
        self
    }
    
    pub fn with_error_reporting(mut self, show_errors: bool) -> Self {
        self.show_errors = show_errors;
        self
    }
}

impl Command for HyperFindFilesCommand {
    fn execute(&self) -> i32 {
        let search_observer: Arc<dyn SearchObserver> = if self.show_progress {
            Arc::new(ProgressObserver::new(self.show_errors))
        } else {
            Arc::new(crate::advanced::NullObserver)
        };
        
        // Create advanced finder with the appropriate filters
        let finder = if let Some(name) = &self.name {
            // Use name and extension filter
            HyperFinderFactory::create_name_and_extension_finder(
                name,
                &self.extension, 
                Some(search_observer)
            )
        } else {
            // Use only extension filter
            HyperFinderFactory::create_extension_finder_with_observer(
                &self.extension, 
                search_observer
            )
        };
        
        // Execute the search
        match finder.find(&self.directory) {
            Ok(files) => {
                if !self.show_progress {
                    if files.is_empty() {
                        if let Some(name) = &self.name {
                            println!("No files with name '{}' and extension '{}' found in '{}'", 
                                name, self.extension, self.directory);
                        } else {
                            println!("No files with extension '{}' found in '{}'", 
                                self.extension, self.directory);
                        }
                    } else {
                        if let Some(name) = &self.name {
                            println!("Found {} file(s) with name '{}' and extension '{}':", 
                                files.len(), name, self.extension);
                        } else {
                            println!("Found {} file(s) with extension '{}':", 
                                files.len(), self.extension);
                        }
                        for file in files {
                            println!("{}", file.display());
                        }
                    }
                } else {
                    // Progress observer already displayed count
                    if !files.is_empty() {
                        println!("\nMatching files:");
                        for file in files {
                            println!("{}", file.display());
                        }
                    }
                }
                0 // Success
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                1 // Error
            }
        }
    }
}

// Original find files command (retained for compatibility)
pub struct FindFilesCommand {
    directory: String,
    extension: String,
    name: Option<String>,
}

impl FindFilesCommand {
    pub fn new(directory: String, extension: String) -> Self {
        Self { 
            directory, 
            extension,
            name: None,
        }
    }
    
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl Command for FindFilesCommand {
    fn execute(&self) -> i32 {
        use crate::finder::FinderFactory;
        
        // Create finder with the appropriate filter
        let finder = if let Some(name) = &self.name {
            FinderFactory::create_name_and_extension_finder(name, &self.extension)
        } else {
            FinderFactory::create_extension_finder(&self.extension)
        };
        
        // Execute the search
        match finder.find(&self.directory) {
            Ok(files) => {
                if files.is_empty() {
                    if let Some(name) = &self.name {
                        println!("No files with name '{}' and extension '{}' found in '{}'", 
                            name, self.extension, self.directory);
                    } else {
                        println!("No files with extension '{}' found in '{}'", 
                            self.extension, self.directory);
                    }
                } else {
                    if let Some(name) = &self.name {
                        println!("Found {} file(s) with name '{}' and extension '{}':", 
                            files.len(), name, self.extension);
                    } else {
                        println!("Found {} file(s) with extension '{}':", 
                            files.len(), self.extension);
                    }
                    for file in files {
                        println!("{}", file.display());
                    }
                }
                0 // Success
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                1 // Error
            }
        }
    }
}

// CLI parser and command executor (using Factory pattern)
pub struct CommandLineParser;

impl CommandLineParser {
    pub fn parse_args(args: &[String]) -> Result<Box<dyn Command>, String> {
        if args.len() < 3 {
            return Err(format!(
                "Usage: {} <directory_path> <file_extension> [options]\n\
                Options:\n\
                  --fast            Use the faster hyper search implementation\n\
                  --progress        Show progress during search\n\
                  --errors          Show directory access errors\n\
                  --name=<value>    Filter by file name (contains search)\n\
                  --standard        Use standard directory traversal\n\
                  --git-aware       Respect .gitignore files (default for --fast)\n\
                  --breadth-first   Use breadth-first traversal\n\n\
                Examples:\n\
                  {} . rs                     Find all .rs files in current directory\n\
                  {} . rs --name=main         Find .rs files containing 'main' in filename\n\
                  {} /path txt --fast         Find all .txt files with high performance search",
                args.get(0).unwrap_or(&"program".to_string()),
                args.get(0).unwrap_or(&"program".to_string()),
                args.get(0).unwrap_or(&"program".to_string()),
                args.get(0).unwrap_or(&"program".to_string())
            ));
        }
        
        let directory = args[1].clone();
        let mut extension = args[2].clone();
        
        // Ensure the extension starts with a dot
        if !extension.starts_with('.') {
            extension = format!(".{}", extension);
        }
        
        // Process optional flags
        let use_hyper = args.iter().any(|arg| arg == "--fast");
        let show_progress = args.iter().any(|arg| arg == "--progress");
        let show_errors = args.iter().any(|arg| arg == "--errors");
        
        // Extract file name if specified
        let name = args.iter()
            .filter_map(|arg| {
                if arg.starts_with("--name=") {
                    Some(arg[7..].to_string())
                } else {
                    None
                }
            })
            .next();
        
        // Determine traversal strategy
        let traversal_strategy = if args.iter().any(|arg| arg == "--standard") {
            TraversalStrategy::Standard
        } else if args.iter().any(|arg| arg == "--git-aware") {
            TraversalStrategy::GitAware
        } else if args.iter().any(|arg| arg == "--breadth-first") {
            TraversalStrategy::BreadthFirst
        } else {
            TraversalStrategy::Standard // Default
        };
        
        if use_hyper || show_progress || name.is_some() {
            // Create hyper command
            let mut command = HyperFindFilesCommand::new(directory, extension)
                .with_traversal_strategy(traversal_strategy)
                .with_progress(show_progress)
                .with_error_reporting(show_errors);
                
            // Add name filter if specified
            if let Some(name_str) = name {
                command = command.with_name(name_str);
            }
                
            Ok(Box::new(command))
        } else {
            // Use original command
            let mut command = FindFilesCommand::new(directory, extension);
            
            // Add name filter if specified
            if let Some(name_str) = name {
                command = command.with_name(name_str);
            }
            
            Ok(Box::new(command))
        }
    }
} 