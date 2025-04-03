mod help;
mod standard_search;
mod advanced_search;

pub use help::HelpCommand;
pub use standard_search::StandardSearchCommand;
pub use advanced_search::AdvancedSearchCommand;

use anyhow::Result;

/// Command interface for different operations
pub trait Command {
    /// Execute the command and return a result
    fn execute(&self) -> Result<()>;
} 