use futures::StreamExt;
use oqab::traverser::Traverser;
use std::path::Path;
use tokio;

#[tokio::main]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let root_path = std::path::Path::new(&args[1]);

    if !root_path.exists() || !root_path.is_dir() {
        eprintln!("Error: '{}' is not a valid directory", root_path.display());
        std::process::exit(1);
    }

    println!("Starting directory traversal of {}...", root_path.display());
    while let Some(result) = traverser.next().await {
        match result {
            Ok(path) => println!("Found: {}", path.display),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    print!("Traversal complete!");
}
