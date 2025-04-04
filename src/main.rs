use std::process;
use anyhow::{Context, Result};
use env_logger::Env;
use log::{error, info, LevelFilter};

use oqab::core::config::FileSearchConfig;
use oqab::commands::{Command, HelpCommand, SearchCommand};

fn main() {
    // Parse command line arguments
    let args = match oqab::cli::args::Args::parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error parsing arguments: {}", err);
            process::exit(1);
        }
    };
    
    // Initialize logger with custom environment based on verbosity flags
    let log_level = if args.silent || args.quiet {
        LevelFilter::Warn
    } else {
        std::env::var("RUST_LOG")
            .map(|level| level.parse().unwrap_or(LevelFilter::Info))
            .unwrap_or(LevelFilter::Info)
    };
    
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .format(|buf, record| {
            use std::io::Write;
            // Simple format without module path or timestamp
            if record.level() <= log::Level::Info {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(buf, "{}: {}", record.level(), record.args())
            }
        })
        .filter(None, log_level)
        .init();
    
    // No longer display the banner here - it will be shown only in help command
    
    // Run the application and handle errors
    if let Err(err) = run(&args) {
        error!("Application error: {:#}", err);
        process::exit(1);
    }
    
    process::exit(0);
}

fn run(args: &oqab::cli::args::Args) -> Result<()> {
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
    
    // Create search command without redundant logging
    if config.advanced_search {
        info!("Using advanced search mode");
    } else {
        info!("Using standard search mode");
    }
    
    Ok(Box::new(SearchCommand::new(config)))
}

/// Display a welcome message - will now only be called from help command
pub fn display_banner() {
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
