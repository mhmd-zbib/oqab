/// Returns the detailed help text for the application
pub fn get_help_text() -> String {
    r#"HyperSearch File Finder

USAGE:
hypersearch [OPTIONS]

OPTIONS:
-h, --help                   Display this help message
-p, --path <DIR>             Directory to search in
-e, --ext <EXT>              File extension to search for (e.g., 'rs' or '.rs')
-n, --name <PATTERN>         Filter by file name pattern
-a, --advanced               Use advanced search algorithm with better performance
-s, --silent                 Suppress progress output
-w, --workers <NUM>          Number of worker threads (default: CPU cores)
-c, --config <FILE>          Load settings from a configuration file
--save-config <FILE>         Save current settings to a configuration file

EXAMPLES:
# Find all Rust files in current directory
hypersearch --path . --ext rs
hypersearch -p . -e rs

# Find files with 'config' in the filename
hypersearch -p . -n config

# Find Rust files with 'main' in the filename
hypersearch -p . -e rs -n main

# Advanced search with silent output
hypersearch -p /path -a -s -e rs

# Use specific number of worker threads
hypersearch -p . -e rs -w 4

# Save search settings to a config file
hypersearch -p . -e rs --save-config myconfig.json

# Use settings from a config file
hypersearch -c myconfig.json

# Load settings but override some options
hypersearch -c myconfig.json -p /different/path
"#.to_string()
} 