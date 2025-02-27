use calamine::{open_workbook, Reader, Xlsx};
use log::{error, info};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::models::SearchResult;

pub fn process_excel_file(
    path: &Path,
    search_name: &str,
    results: &Arc<Mutex<Vec<SearchResult>>>,
) -> Result<(), String> {
    info!("Processing file: {}", path.display());

    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(wb) => wb,
        Err(e) => {
            error!("Failed to open workbook {}: {}", path.display(), e);
            return Err(e.to_string());
        }
    };

    let sheet_names = workbook.sheet_names().to_owned();
    let mut local_results = Vec::new();

    for sheet in sheet_names {
        if let Some(Ok(range)) = workbook.worksheet_range(&sheet) {
            for (row_idx, row) in range.rows().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    if cell.to_string().contains(search_name) {
                        local_results.push(SearchResult {
                            file_path: path.to_string_lossy().to_string(),
                            column: (col_idx + 1) as u32,
                            row: (row_idx + 1) as u32,
                        });
                    }
                }
            }
        }
    }

    if !local_results.is_empty() {
        let mut global_results = results.lock().unwrap();
        global_results.extend(local_results);
    }

    Ok(())
}
