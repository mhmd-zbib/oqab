use log::error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn validate_path(path: &Path) -> bool {
    path.exists()
}

pub fn collect_excel_files(dir_path: &Path, max_depth: u32) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_excel_files_recursive(dir_path, &mut files, 0, max_depth);
    files
}

fn collect_excel_files_recursive(
    dir_path: &Path,
    files: &mut Vec<PathBuf>,
    depth: u32,
    max_depth: u32,
) {
    if depth >= max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("xlsx") {
                files.push(path);
            } else if path.is_dir() {
                collect_excel_files_recursive(&path, files, depth + 1, max_depth);
            }
        }
    }
}
