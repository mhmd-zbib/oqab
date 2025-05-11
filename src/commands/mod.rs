mod help;
mod search;
mod grep;

pub use help::HelpCommand;
pub use search::SearchCommand;
pub use grep::GrepCommand;

use anyhow::Result;

/// Command interface for different operations
pub trait Command {
    /// Execute the command and return a result
    fn execute(&self) -> Result<()>;
} 