/// Returns the detailed help text for the application
pub fn get_help_text() -> String {
    r#"Oqab File Finder

USAGE:
oqab [OPTIONS]

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
oqab --path . --ext rs
oqab -p . -e rs

# Find files with 'config' in the filename
oqab -p . -n config

# Find Rust files with 'main' in the filename
oqab -p . -e rs -n main

# Advanced search with silent output
oqab -p /path -a -s -e rs

# Use specific number of worker threads
oqab -p . -e rs -w 4

# Save search settings to a config file
oqab -p . -e rs --save-config myconfig.json

# Use settings from a config file
oqab -c myconfig.json

# Load settings but override some options
oqab -c myconfig.json -p /different/path
"#.to_string()
} 