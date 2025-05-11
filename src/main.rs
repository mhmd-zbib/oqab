use std::process;
use anyhow::{Context, Result};
use env_logger::Env;
use log::{error, info, warn, LevelFilter};

use oqab::core::{ConfigManager, FileSearchConfig, Platform};
use oqab::commands::{Command, HelpCommand, SearchCommand, GrepCommand, FuzzyCommand};

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
    
    // Run the application and handle errors
    if let Err(err) = run(&args) {
        error!("Application error: {:#}", err);
        process::exit(1);
    }
    
    process::exit(0);
}

fn run(args: &oqab::cli::args::Args) -> Result<()> {
    // Process arguments into a configuration
    let mut config = args.process()
        .context("Failed to process arguments into a valid configuration")?;
    
    // Check if help is requested
    let showing_help = args.help || (config.file_extension.is_none() && config.file_name.is_none() && config.pattern.is_none());
    
    // Set root directory as default search path if none specified (but not when showing help)
    if config.path.is_none() && !showing_help {
        let root_path = Platform::root_directory().to_string_lossy().to_string();
        warn!("No path specified. Searching from root directory ({}). This may take a long time and require elevated permissions.", root_path);
        config.path = Some(root_path);
    } else if let Some(path) = &config.path {
        if Platform::is_root_path(path) && !showing_help {
            warn!("Searching from root directory. This may take a long time and require elevated permissions.");
        }
    }
    
    // Initialize the singleton configuration manager
    ConfigManager::instance().initialize(config.clone());
    
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
    // Display help if explicitly requested or if no search criteria provided
    if config.help || (config.file_extension.is_none() && config.file_name.is_none() && config.pattern.is_none()) {
        return Ok(Box::new(HelpCommand::new()));
    }
    
    // If a pattern is specified, use the GrepCommand for text search
    if config.pattern.is_some() {
        info!("Using text pattern search mode");
        return Ok(Box::new(GrepCommand::new(config)));
    }
    
    // If fuzzy search is enabled, use the FuzzyCommand
    if config.fuzzy {
        info!("Using fuzzy search mode");
        return Ok(Box::new(FuzzyCommand::new(config)));
    }
    
    // Otherwise, use the standard file search
    info!("Using standard search mode");
    Ok(Box::new(SearchCommand::new(config)))
}
