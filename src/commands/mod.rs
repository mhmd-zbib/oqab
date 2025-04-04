mod help;
mod search;

pub use help::HelpCommand;
pub use search::SearchCommand;

use anyhow::Result;

/// Command interface for different operations
pub trait Command {
    /// Execute the command and return a result
    fn execute(&self) -> Result<()>;
} 