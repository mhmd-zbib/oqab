use std::process;
use anyhow::{Context, Result};
use env_logger::Env;
use log::{error, info};

use oqab::cli::args::Args;
use oqab::core::config::FileSearchConfig;
use oqab::commands::{Command, HelpCommand, SearchCommand};

fn main() {
    // Initialize logger with custom environment
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    
    // Display application banner
    display_banner();
    
    // Run the application and handle errors
    if let Err(err) = run() {
        error!("Application error: {:#}", err);
        process::exit(1);
    }
    
    process::exit(0);
}

fn run() -> Result<()> {
    // Parse command line arguments using clap
    let args = Args::parse()
        .context("Failed to parse command line arguments")?;
    
    // Process arguments into a configuration
    let config = args.process()
        .context("Failed to process arguments into a valid configuration")?;
    
    // Save configuration if requested
    if args.save_config_file.is_some() {
        args.save_config(&config)
            .context("Failed to save configuration to file")?;
        info!("Configuration saved successfully");
    }
    
    // Create and execute the appropriate command
    create_command(&config)?
        .execute()
        .context("Command execution failed")?;
    
    Ok(())
}

/// Create the appropriate command based on the configuration
fn create_command(config: &FileSearchConfig) -> Result<Box<dyn Command + '_>> {
    // Display help if no search criteria provided
    if config.file_extension.is_none() && config.file_name.is_none() {
        return Ok(Box::new(HelpCommand::new()));
    }
    
    // Create search command
    info!("Using {} search mode", 
        if config.advanced_search { "advanced" } else { "standard" });
    
    Ok(Box::new(SearchCommand::new(config)))
}

/// Display a welcome message
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
