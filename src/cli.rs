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
    max_depth: Option<usize>,
    skip_hidden: bool,
    follow_links: bool,
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
            max_depth: None,
            skip_hidden: false,
            follow_links: true,
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
    
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }
    
    pub fn with_skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }
    
    pub fn with_follow_links(mut self, follow: bool) -> Self {
        self.follow_links = follow;
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
    max_depth: Option<usize>,
    skip_hidden: bool,
}

impl FindFilesCommand {
    pub fn new(directory: String, extension: String) -> Self {
        Self { 
            directory, 
            extension,
            name: None,
            max_depth: None,
            skip_hidden: false,
        }
    }
    
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }
    
    pub fn with_skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }
}

impl Command for FindFilesCommand {
    fn execute(&self) -> i32 {
        use crate::finder::FinderFactory;
        
        // Print search configuration
        println!("🔍 Standard File Finder");
        println!("-----------------------");
        if self.max_depth.is_some() || self.skip_hidden {
            println!("Search options:");
            if let Some(depth) = self.max_depth {
                println!("  - Max depth: {}", depth);
            }
            if self.skip_hidden {
                println!("  - Skipping hidden files/directories");
            }
            println!();
        }
        
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
                  -f, --fast          Use the faster hyper search implementation\n\
                  -p, --progress      Show progress during search\n\
                  -e, --errors        Show directory access errors\n\
                  -n, --name <value>  Filter by file name (contains search)\n\
                  -r, --recursive     Search directories recursively (default: true)\n\
                  -d, --depth <num>   Maximum directory depth to search (default: unlimited)\n\
                  --no-hidden         Skip hidden files and directories\n\
                  --no-follow-links   Don't follow symbolic links\n\
                  --traversal <type>  Directory traversal strategy:\n\
                                       - standard     Default traversal\n\
                                       - git-aware    Respect .gitignore files (default for --fast)\n\
                                       - breadth-first Use breadth-first traversal\n\n\
                Examples:\n\
                  {} . rs                     Find all .rs files in current directory\n\
                  {} . rs -n main             Find .rs files containing 'main' in filename\n\
                  {} /path txt -f -p          Find all .txt files with fast search and progress\n\
                  {} . rs --depth 2 --no-hidden  Only search 2 levels deep, skip hidden files",
                args.get(0).unwrap_or(&"program".to_string()),
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
        
        // Process optional flags with both short and long forms
        let use_hyper = args.iter().any(|arg| arg == "--fast" || arg == "-f");
        let show_progress = args.iter().any(|arg| arg == "--progress" || arg == "-p");
        let show_errors = args.iter().any(|arg| arg == "--errors" || arg == "-e");
        let skip_hidden = args.iter().any(|arg| arg == "--no-hidden");
        let no_follow_links = args.iter().any(|arg| arg == "--no-follow-links");
        
        // Parse name parameter (supports both formats: --name=value and --name value)
        let name = Self::extract_parameter_value(args, &["-n", "--name"]);
        
        // Parse depth parameter
        let depth = Self::extract_parameter_value(args, &["-d", "--depth"])
            .and_then(|d| d.parse::<usize>().ok());
            
        // Determine traversal strategy
        let traversal_strategy = if let Some(strategy) = Self::extract_parameter_value(args, &["--traversal"]) {
            match strategy.as_str() {
                "standard" => TraversalStrategy::Standard,
                "git-aware" => TraversalStrategy::GitAware,
                "breadth-first" => TraversalStrategy::BreadthFirst,
                _ => TraversalStrategy::Standard,
            }
        } else if args.iter().any(|arg| arg == "--standard") {
            TraversalStrategy::Standard
        } else if args.iter().any(|arg| arg == "--git-aware") {
            TraversalStrategy::GitAware
        } else if args.iter().any(|arg| arg == "--breadth-first") {
            TraversalStrategy::BreadthFirst
        } else {
            TraversalStrategy::Standard // Default
        };
        
        if use_hyper || show_progress || name.is_some() {
            // Create hyper command with all options
            let mut command = HyperFindFilesCommand::new(directory, extension)
                .with_traversal_strategy(traversal_strategy)
                .with_progress(show_progress)
                .with_error_reporting(show_errors)
                .with_max_depth(depth)
                .with_skip_hidden(skip_hidden)
                .with_follow_links(!no_follow_links);
                
            // Add name filter if specified
            if let Some(name_str) = name {
                command = command.with_name(name_str);
            }
                
            Ok(Box::new(command))
        } else {
            // Use original command - also with new options
            let mut command = FindFilesCommand::new(directory, extension)
                .with_max_depth(depth)
                .with_skip_hidden(skip_hidden);
            
            // Add name filter if specified
            if let Some(name_str) = name {
                command = command.with_name(name_str);
            }
            
            Ok(Box::new(command))
        }
    }
    
    // Helper method to extract parameter values from command line arguments
    fn extract_parameter_value(args: &[String], param_flags: &[&str]) -> Option<String> {
        // First check for combined format like --name=value
        for arg in args {
            for &flag in param_flags {
                if arg.starts_with(&format!("{}=", flag)) {
                    return Some(arg[flag.len() + 1..].to_string());
                }
            }
        }
        
        // Then check for separated format like --name value
        for i in 0..args.len() - 1 {
            if param_flags.contains(&args[i].as_str()) {
                return Some(args[i + 1].clone());
            }
        }
        
        None
    }
} 