use anyhow::Result;
use console::style;
use crate::commands::Command;

/// Command for displaying help information
pub struct HelpCommand;

impl HelpCommand {
    /// Create a new help command
    pub fn new() -> Self {
        Self
    }
    
    /// Display the application banner
    fn display_banner() {
        println!("  ____                 _     ");
        println!(" / __ \\               | |    ");
        println!("| |  | | __ _  __ _ _ | |__  ");
        println!("| |  | |/ _` |/ _` | '| '_ \\ ");
        println!("| |__| | (_| | (_| | || |_) |");
        println!(" \\____/ \\__, |\\__,_|_||_.__/ ");
        println!("         __/ |                ");
        println!("        |___/  Search Utility ");
        println!();
    }
}

impl Default for HelpCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for HelpCommand {
    fn execute(&self) -> Result<()> {
        // Display banner first
        Self::display_banner();
        
        // Then display help text with colorful formatting
        // We're implementing our own colorful help text directly
        // instead of using the get_help_text() function
        
        // Print usage section with colors
        println!("{}", style("USAGE:").bold().green());
        println!("oqab [QUERY]");
        println!("oqab [OPTIONS] [QUERY]");
        println!("oqab --grep PATTERN [OPTIONS]
");
        
        // Print options section with colors
        println!("{}", style("OPTIONS:").bold().green());
        println!("{} Display this help message", style("-h, --help                  ").yellow());
        println!("{} Directory to search in (default: root directory)", style("-p, --path <DIR>            ").yellow());
        println!("{} File extension to search for (e.g., 'rs' or '.rs')", style("-e, --ext <EXT>             ").yellow());
        println!("{} Filter by file name pattern", style("-n, --name <PATTERN>        ").yellow());
        println!("{} Search for text pattern within files (grep-like functionality)", style("-g, --grep <PATTERN>        ").yellow());
        println!("{} Case insensitive search", style("-i, --ignore-case          ").yellow());
        println!("{} Show line numbers in search results", style("--line-number               ").yellow());
        println!("{} Show only filenames of files containing the pattern", style("--files-with-matches        ").yellow());
        println!("{} Suppress progress output", style("-s, --silent                ").yellow());
        println!("{} Quiet mode (less verbose output)", style("-q, --quiet                 ").yellow());
        println!("{} Number of worker threads (default: CPU cores)", style("-w, --workers <NUM>         ").yellow());
        println!("{} Load settings from a configuration file", style("-c, --config <FILE>         ").yellow());
        println!("{} Save current settings to a configuration file
", style("--save-config <FILE>        ").yellow());
        
        // Print examples section with colors
        println!("{}", style("EXAMPLES:").bold().green());
        println!("# Simple file search by name (searches from root directory)");
        println!("{}", style("oqab main.rs").italic());
        println!();
        println!("# Find all Rust files in current directory");
        println!("{}", style("oqab --path . --ext rs").italic());
        println!("{}", style("oqab -p . -e rs").italic());
        println!();
        println!("# Find files with 'config' in the filename");
        println!("{}", style("oqab -p . -n config").italic());
        println!();
        println!("# Search for text within files (grep-like functionality)");
        println!("{}", style("oqab --grep \"function\" -p .").italic());
        println!();
        println!("# Case-insensitive search with line numbers");
        println!("{}", style("oqab --grep \"error\" --ignore-case --line-number").italic());
        println!();
        println!("# Search for text only in specific file types");
        println!("{}", style("oqab --grep \"import\" --ext py").italic());
        println!();
        println!("# Show only filenames containing matches");
        println!("{}", style("oqab --grep \"TODO\" --files-with-matches").italic());
        println!();
        println!("# Save search settings to a config file");
        println!("{}", style("oqab -p . -e rs --save-config myconfig.json").italic());
        println!();
        println!("# Use settings from a config file");
        println!("{}", style("oqab -c myconfig.json").italic());
        
        Ok(())
    }
} 