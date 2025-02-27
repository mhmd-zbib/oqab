use env_logger::Env;
use log::{error, info};
use std::process;

use oqab::cli::build_cli;
use oqab::file_system::validate_path;
use oqab::search_engine::search_excel_files;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = build_cli().get_matches();

    let input_path = matches
        .get_one::<String>("input")
        .expect("Input path is required");
    let search_target = matches
        .get_one::<String>("target")
        .expect("Search target is required");
    let max_depth: u32 = matches
        .get_one::<String>("max-depth")
        .unwrap()
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("Error: Max depth must be a positive integer");
            process::exit(1);
        });
    let thread_count: usize = matches
        .get_one::<String>("threads")
        .unwrap()
        .parse()
        .unwrap_or(0);

    if thread_count > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .unwrap();
        info!("Using {} threads", thread_count);
    } else {
        info!("Using default thread count ({})", num_cpus::get());
    }

    let path = std::path::Path::new(input_path);
    if !validate_path(path) {
        error!("Path does not exist: {}", input_path);
        process::exit(1);
    }

    info!("Processing path: {}", input_path);
    info!("Searching for: {}", search_target);

    match search_excel_files(path, search_target, max_depth) {
        Ok(results) => {
            if results.is_empty() {
                println!("No matches found");
            } else {
                for result in results {
                    println!("Found in: {}", result.file_path);
                    println!("Location: Column {}, Row {}", result.column, result.row);
                }
            }
        }
        Err(e) => {
            error!("Error: {}", e);
            process::exit(1);
        }
    }
}