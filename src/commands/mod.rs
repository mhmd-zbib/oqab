mod help;
mod search;
mod grep;
mod fuzzy;

pub use help::HelpCommand;
pub use search::SearchCommand;
pub use grep::GrepCommand;
pub use fuzzy::FuzzyCommand;

use anyhow::Result;

/// Command interface for different operations
pub trait Command {
    /// Execute the command and return a result
    fn execute(&self) -> Result<()>;
} 