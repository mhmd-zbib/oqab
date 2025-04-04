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
}

impl Default for HelpCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for HelpCommand {
    fn execute(&self) -> Result<()> {
        // Display banner first
        crate::display_banner();
        
        // Then display help text
        println!("{}", get_help_text());
        
        Ok(())
    }
} 