use calamine::{open_workbook, Reader, Xlsx};
use csv::Writer;
use rayon::prelude::*;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
    time::Instant,
    collections::HashMap,
};
use walkdir;
use regex::Regex;

fn convert_excels_to_csv(path: &str, output_csv: &str) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    println!("[LOG] Started Excel to CSV conversion process");

    let files = get_excel_files(path);

    let write_start = Instant::now();
    println!("[LOG] Started creating CSV file: {}", output_csv);
    let mut writer = Writer::from_path(output_csv)?;
    writer.write_record(&["file_path", "sheet_name", "row_number", "content"])?;
    println!(
        "[LOG] Finished creating CSV file: {} in {:?}",
        output_csv,
        write_start.elapsed()
    );

    let mut total_rows = 0;
    for file in files {
        let file_start = Instant::now();
        println!("[LOG] Started processing file: {}", file.display());

        let mut workbook: Xlsx<_> = open_workbook(&file)?;
        let sheet_names = workbook.sheet_names().to_vec();

        for sheet_name in sheet_names {
            let sheet_start = Instant::now();
            println!("[LOG] Started processing sheet: {}", sheet_name);

            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                let row_count = range.rows().count();
                total_rows += row_count;

                range.rows().enumerate().for_each(|(row_idx, row)| {
                    let content: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
                    writer.write_record(&[
                        file.to_string_lossy().to_string(),
                        sheet_name.clone(),
                        (row_idx + 1).to_string(),
                        content.join(" | "),
                    ]).unwrap();
                });

                println!(
                    "[LOG] Finished processing sheet: {} in {:?} ({} rows)",
                    sheet_name,
                    sheet_start.elapsed(),
                    row_count
                );
            }
        }

        println!(
            "[LOG] Finished processing file: {} in {:?}",
            file.display(),
            file_start.elapsed()
        );
    }

    let flush_start = Instant::now();
    println!("[LOG] Started flushing CSV data");
    writer.flush()?;
    println!(
        "[LOG] Finished flushing CSV data in {:?}",
        flush_start.elapsed()
    );

    println!(
        "[LOG] Finished Excel to CSV conversion process in {:?}. Total rows processed: {}",
        total_start.elapsed(),
        total_rows
    );
    Ok(())
}

struct CsvIndex {
    records: Vec<csv::StringRecord>,
    content_index: HashMap<String, Vec<usize>>,
}

impl CsvIndex {
    fn new(csv_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_path(csv_path)?;
        let mut records = Vec::new();
        let mut content_index = HashMap::new();

        for (idx, result) in rdr.records().enumerate() {
            let record = result?;
            records.push(record.clone());
            
            // Index content for faster search
            let content = record[3].to_lowercase();
            content_index.entry(content).or_insert_with(Vec::new).push(idx);
        }

        Ok(Self { records, content_index })
    }

    fn search(&self, search_term: &str) -> Vec<(String, String, String)> {
        let search_term = search_term.to_lowercase();
        
        self.content_index.par_iter()
            .filter(|(content, _)| content.contains(&search_term))
            .flat_map(|(_, indices)| indices)
            .map(|&idx| {
                let record = &self.records[idx];
                (
                    record[0].to_string(),
                    record[1].to_string(),
                    record[2].to_string(),
                )
            })
            .collect()
    }

    fn regex_search(&self, pattern: &str) -> Result<Vec<(String, String, String)>, regex::Error> {
        let re = Regex::new(pattern)?;
        
        Ok(self.records.par_iter()
            .filter(|record| re.is_match(&record[3]))
            .map(|record| {
                (
                    record[0].to_string(),
                    record[1].to_string(),
                    record[2].to_string(),
                )
            })
            .collect())
    }
}

fn main() {
    let path = "C:/Development";
    let search_term = "Janusik";
    let csv_path = "combined_data.csv";

    // Convert Excel files to CSV
    if !Path::new(csv_path).exists() {
        if let Err(e) = convert_excels_to_csv(path, csv_path) {
            eprintln!("Error converting Excel files: {}", e);
            return;
        }
    }

    // Create index
    let index = match CsvIndex::new(csv_path) {
        Ok(index) => index,
        Err(e) => {
            eprintln!("Error creating index: {}", e);
            return;
        }
    };

    // Search using index
    let results = index.search(search_term);
    
    // Regex search example
    let regex_results = match index.regex_search(r"\bJ\w+ik\b") {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Regex error: {}", e);
            return;
        }
    };

    // Output results
    if results.is_empty() {
        println!("No matches found.");
    } else {
        println!("Found matches:");
        for (file_path, sheet_name, row_number) in results {
            println!(
                "File: {}\nSheet: {}\nRow: {}\n",
                file_path, sheet_name, row_number
            );
        }
    }
}

fn get_excel_files(path: &str) -> Vec<PathBuf> {
    let total_start = Instant::now();
    println!("[LOG] Started searching in directory: {}", path);

    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .min_depth(1)
        .max_depth(5)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| {
            let entry = e.ok()?;
            let path = entry.path();
            
            if path.is_dir() {
                let dir_start = Instant::now();
                println!("[LOG] Started searching directory: {}", path.display());
                println!(
                    "[LOG] Finished searching directory: {} in {:?}",
                    path.display(),
                    dir_start.elapsed()
                );
                None
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "xlsx") {
                println!("[LOG] Found Excel file: {}", path.display());
                Some(path.to_path_buf())
            } else {
                None
            }
        });

    files.extend(walker);
    println!(
        "[LOG] Finished searching in directory: {} in {:?}. Found {} Excel files.",
        path,
        total_start.elapsed(),
        files.len()
    );
    files
}
