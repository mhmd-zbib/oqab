// Command pattern
pub trait Command {
    fn execute(&self) -> i32;
}

// Find files command implementation
pub struct FindFilesCommand {
    directory: String,
    extension: String,
}

impl FindFilesCommand {
    pub fn new(directory: String, extension: String) -> Self {
        Self { directory, extension }
    }
}

impl Command for FindFilesCommand {
    fn execute(&self) -> i32 {
        use crate::finder::FinderFactory;
        
        // Create finder with the extension filter
        let finder = FinderFactory::create_extension_finder(&self.extension);
        
        // Execute the search
        match finder.find(&self.directory) {
            Ok(files) => {
                if files.is_empty() {
                    println!("No files with extension '{}' found in '{}'", self.extension, self.directory);
                } else {
                    println!("Found {} file(s) with extension '{}':", files.len(), self.extension);
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
                "Usage: {} <directory_path> <file_extension>\nExample: {} /path/to/search .pdf", 
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
        
        Ok(Box::new(FindFilesCommand::new(directory, extension)))
    }
} 