use oqab::traverser::{Traverser, TraverserConfig};
use std::path::PathBuf;
use tokio::stream::StreamExt;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let root_path = PathBuf::from("C:/Users");
    
    let config = TraverserConfig {
        max_depth: Some(3),
        follow_symlinks: true,
        exclude_patterns: vec![
            "AppData".to_string(),
            "Local".to_string()
        ],
    };
    
    println!("Starting directory traversal of {}...", root_path.display());
    let mut traverser = Traverser::new(root_path, config);
    
    while let Some(result) = traverser.next().await {
        match result {
            Ok(path) => println!("Found: {}", path.display()),
            Err(e) if !matches!(e, TraverserError::ChannelClosed) => eprintln!("Error: {}", e),
            _ => (),
        }
    }
    
    println!("Traversal complete!");
    if let Some(duration) = traverser.elapsed_time() {
        println!("Time taken: {:?}", duration);
    } else {
        eprintln!("Error: Could not determine traversal time");
    }
} 