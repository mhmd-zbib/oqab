use anyhow::Result;
use log::{info, debug};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::path::PathBuf;
use std::time::Instant;

use crate::commands::Command;
use crate::core::config::FileSearchConfig;
use crate::core::observer::NullObserver;
use crate::utils::standard_search;



/// Command for fuzzy file searching
pub struct FuzzyCommand<'a> {
    config: &'a FileSearchConfig,
}

impl<'a> FuzzyCommand<'a> {
    /// Create a new fuzzy search command
    pub fn new(config: &'a FileSearchConfig) -> Self {
        Self { config }
    }

    /// Process files with fuzzy matching
    fn process_files(&self, files: &[PathBuf]) -> Result<()> {
        // Create a fuzzy matcher with appropriate settings
        let matcher = SkimMatcherV2::default();
        
        // Get the search pattern
        let pattern = if let Some(name) = &self.config.file_name {
            name
        } else {
            // If no pattern specified, nothing to match against
            return Ok(());
        };
        
        // Get threshold from config or use default
        let threshold = self.config.fuzzy_threshold.unwrap_or(50) as i64;
        
        // Track matches for sorting by score
        let mut matches = Vec::new();
        
        // Process each file
        for file_path in files {
            let file_name = file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            // Perform fuzzy matching
            if let Some(score) = matcher.fuzzy_match(file_name, pattern) {
                // Only include matches that meet the threshold
                if score > threshold {
                    matches.push((file_path.clone(), score));
                }
            }
        }
        
        // Sort matches by score (highest first)
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Display results
        if !matches.is_empty() {
            println!("Found {} fuzzy matching file(s):", matches.len());
            for (path, score) in matches {
                // Calculate match quality as a percentage (0-100)
                let quality = ((score as f64) / 100.0).min(1.0) * 100.0;
                println!("  {} (match quality: {:.0}%)", path.display(), quality);
            }
        } else {
            println!("No fuzzy matches found.");
        }
        
        Ok(())
    }
}

impl<'a> Command for FuzzyCommand<'a> {
    fn execute(&self) -> Result<()> {
        let start_time = Instant::now();
        let search_path = PathBuf::from(self.config.get_path());
        info!("Starting fuzzy search in {}", search_path.display());
        
        // Use standard search to collect files, then apply fuzzy matching
        let results = standard_search::search_directory(
            &search_path,
            self.config,
            &NullObserver,
        )?;
        
        debug!("Found {} files to process for fuzzy matching", results.len());
        
        // Process the collected files with fuzzy matching
        self.process_files(&results)?;
        
        // Display performance metrics
        let elapsed = start_time.elapsed();
        println!("\nPerformance:");
        println!("  Time taken: {:.2} seconds", elapsed.as_secs_f64());
        println!("  Files processed: {}", results.len());
        
        Ok(())
    }
}
