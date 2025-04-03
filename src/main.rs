use std::process;
use anyhow::{Context, Result};
use env_logger::Env;
use log::error;

use oqab::cli::args::Args;
use oqab::config::FileSearchConfig;
use oqab::commands::{Command, HelpCommand, StandardSearchCommand, AdvancedSearchCommand};

fn main() {
    // Initialize logger with custom environment
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    
    // Display application banner
    display_banner();
    
    // Run the application and handle errors
    if let Err(err) = run() {
        error!("Error: {}", err);
        process::exit(1);
    }
    
    process::exit(0);
}

fn run() -> Result<()> {
    // Parse command line arguments using clap
    let args = Args::parse().context("Failed to parse command line arguments")?;
    
    // Process arguments into a configuration
    let config = args.process().context("Failed to process arguments")?;
    
    // Create and execute the appropriate command
    create_command(&config)?.execute()
}

/// Create the appropriate command based on the configuration
fn create_command(config: &FileSearchConfig) -> Result<Box<dyn Command>> {
    // Display help if no search criteria provided
    if config.file_extension.is_none() && config.file_name.is_none() {
        return Ok(Box::new(HelpCommand::new()));
    }
    
    // Create the appropriate command based on config
    if config.advanced_search {
        // Advanced search with observer
        Ok(Box::new(AdvancedSearchCommand::new(config.clone())))
    } else {
        // Standard search
        Ok(Box::new(StandardSearchCommand::new(config.clone())))
    }
}

/// Display a welcome message
fn display_banner() {
    println!(" _   _                      _____                     _     ");
    println!("| | | |                    / ____|                   | |    ");
    println!("| |_| |_   _ _ __   ___  _| (___   ___  __ _ _ __ ___| |__  ");
    println!("|  _  | | | | '_ \\ / _ \\| |\\___ \\ / _ \\/ _` | '__/ __| '_ \\ ");
    println!("| | | | |_| | |_) | (_) | |____) |  __/ (_| | | | (__| | | |");
    println!("|_| |_|\\__, | .__/ \\___/|_|_____/ \\___|\\__,_|_|  \\___|_| |_|");
    println!("        __/ | |                                             ");
    println!("       |___/|_|  File Finder                                ");
    println!();
}
