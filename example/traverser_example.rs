use futures::StreamExt;
use oqab::traverser::{Traverser, TraverserConfig};
use std::path::Path;
use tokio;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let root_path = Path::new(&args[1]);

    if !root_path.exists() || !root_path.is_dir() {
        eprintln!("Error: '{}' is not a valid directory", root_path.display());
        std::process::exit(1);
    }

    let config = TraverserConfig {
        max_depth: None,
        follow_symlinks: false,
        exclude_patterns: vec![
            String::from(".git"),
            String::from("target"),
            String::from("node_modules"),
        ],
    };

    let mut traverser = Traverser::new(root_path, config);

    println!("Starting directory traversal of {}...", root_path.display());

    while let Some(result) = traverser.next().await {
        match result {
            Ok(path) => println!("Found: {}", path.display()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Traversal complete!");
}
