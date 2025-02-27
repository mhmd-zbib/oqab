use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::excel_processor::process_excel_file;
use crate::file_system::collect_excel_files;
use crate::models::SearchResult;

/// Main search function that scans Excel files for a specific term
pub fn search_excel_files(
    path: &Path,
    search_name: &str,
    max_depth: u32,
) -> Result<Vec<SearchResult>, String> {
    let results = Arc::new(Mutex::new(Vec::new()));

    if path.is_file() {
        if path.extension().and_then(|s| s.to_str()) == Some("xlsx") {
            process_excel_file(path, search_name, &results)?;
        }
    } else if path.is_dir() {
        // First, collect all Excel files
        let excel_files = collect_excel_files(path, max_depth);

        // Process files in parallel
        excel_files.par_iter().for_each(|file_path| {
            let _ = process_excel_file(file_path, search_name, &results);
        });
    }

    Ok(Arc::try_unwrap(results).unwrap().into_inner().unwrap())
}