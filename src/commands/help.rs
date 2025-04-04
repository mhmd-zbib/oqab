use anyhow::Result;
use crate::commands::Command;
use crate::cli::help_text::get_help_text;

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
        println!("        |___/  File Finder    ");
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
        
        // Then display help text
        println!("{}", get_help_text());
        
        Ok(())
    }
} 