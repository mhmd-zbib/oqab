# Oqab Search Utility

A high-performance search utility that makes finding information across multiple files as easy as an eagle spotting its prey. Whether you're searching through code, documents, or data files, Oqab (Arabic for "eagle") provides efficient content and file searching across complex directory structures.

## Why Oqab?

Born from the real-world need to search through large numbers of files in deeply nested directories, Oqab makes this task effortless. Whether you're searching for specific content across multiple files or looking for files with particular patterns, Oqab has got you covered.

## Status: Work in Progress 🚧

While the basic file finding functionality is implemented, we're actively working on these crucial features:

### Coming Soon
- **Enhanced Content Search**:
  - Support for various file formats
  - Pattern matching and regex support for content
  - Results highlighting and export
  
- **Advanced Directory Processing**:
  - Smart multi-processing for nested directories
  - Intelligent workload distribution
  - Progress tracking for deep directory structures
  
- **Performance Optimization**:
  - Smart caching system for frequently accessed directories
  - Cache invalidation strategies
  - Memory-efficient file content scanning

## Overview

Oqab is a command-line utility that provides advanced searching capabilities with a focus on performance, flexibility, and usability. It allows users to quickly search through file contents and locate files based on various criteria including content patterns, file extension, name pattern, file size, and modification date.

## Key Features

- **Multi-threaded Processing**: Leverages parallel execution for significantly faster searching
- **Comprehensive Search Capabilities**:
  - Content-based search within files
  - Pattern matching in file contents
  - Regular expression support
  - Case-sensitive/insensitive search options
- **Dual Search Modes**:
  - Standard search for common use cases
  - Advanced search with optimized worker pools for handling large file systems
- **Comprehensive Filtering**:
  - Content pattern matching
  - Extension-based filtering
  - Name pattern matching
  - Size constraints (minimum/maximum)
  - Date-based filtering (newer than/older than)
- **Flexible Configuration**:
  - Command-line interface for direct usage
  - JSON configuration files for reusable search profiles
- **Performance Metrics**: Detailed statistics about search operations
- **Robust Error Handling**: Comprehensive error detection and reporting
- **Symlink Support**: Option to follow or ignore symbolic links

## Installation

### Prerequisites

- Rust toolchain (1.56.0 or newer)
- Cargo package manager

### Building from Source

```bash
# Clone the repository
git clone https://github.com/username/oqab.git
cd oqab

# Build the project
cargo build --release

# The binary will be available at target/release/oqab
```

## Usage Examples

### Basic Usage

Search for content in all files:
```bash
oqab --search "pattern"
```

Search for content in specific file types:
```bash
oqab --search "pattern" --ext txt
```

Find files containing specific content in a directory:
```bash
oqab --path /path/to/search --search "pattern" --ext rs
```

Case-insensitive content search:
```bash
oqab --search "pattern" --ignore-case
```

### Advanced Filtering

Find large files (> 1MB):
```bash
oqab --path . --min-size 1MB
```

Find recently modified files:
```bash
oqab --path . --newer-than 2023-01-01
```

Combined filters (Rust files with "test" in the name):
```bash
oqab --path . --ext rs --name test
```

### Performance Options

Use advanced search algorithm for better performance:
```bash
oqab --path . --ext log --advanced
```

Specify number of worker threads:
```bash
oqab --path . --ext rs --workers 8
```

Run in silent mode (no progress output):
```bash
oqab --path . --ext rs --silent
```

## Command Line Reference

```
USAGE:
oqab [OPTIONS]

OPTIONS:
  -h, --help                   Display this help message
  -p, --path <DIR>             Directory to search in
  -s, --search <PATTERN>       Content pattern to search for
  -i, --ignore-case            Perform case-insensitive search
  -e, --ext <EXT>              File extension to search for (e.g., 'rs' or '.rs')
  -n, --name <PATTERN>         Filter by file name pattern
  --min-size <SIZE>            Minimum file size (e.g., '10kb', '1MB')
  --max-size <SIZE>            Maximum file size
  --newer-than <DATE>          Files newer than specified date (YYYY-MM-DD)
  --older-than <DATE>          Files older than specified date (YYYY-MM-DD)
  -a, --advanced               Use advanced search algorithm with better performance
  -s, --silent                 Suppress progress output
  -w, --workers <NUM>          Number of worker threads (default: CPU cores)
  -r, --recursive              Search recursively in subdirectories
  --follow-links               Follow symbolic links
  -c, --config <FILE>          Load settings from a configuration file
  --save-config <FILE>         Save current settings to a configuration file
```

## Configuration Files

Oqab supports JSON configuration files for storing and reusing search settings.

Example configuration file:
```json
{
  "path": "/home/user/projects",
  "search_pattern": "TODO",
  "ignore_case": true,
  "file_extension": "rs",
  "file_name": "test",
  "advanced_search": true,
  "thread_count": 4,
  "show_progress": true,
  "recursive": true,
  "follow_symlinks": false,
  "min_size": 1024,
  "max_size": null,
  "newer_than": "2023-01-01",
  "older_than": null
}
```

## Architecture

Oqab is built with a focus on maintainable and efficient code using several design patterns:

- **Observer Pattern**: For tracking and reporting search progress
- **Strategy Pattern**: For interchangeable search and traversal algorithms
- **Factory Pattern**: For creating appropriate searchers based on configuration
- **Builder Pattern**: For constructing complex search configurations
- **Filter Chain**: For combining multiple file filters

## Performance

The application includes two search modes:

1. **Standard Search**: Efficient for most use cases and smaller directory structures
2. **Advanced Search**: Uses a worker pool with optimized thread management for handling very large directory structures (100,000+ files)

Performance metrics are displayed after each search operation, showing:
- Time taken
- Files processed
- Directories traversed
- Processing rate (files/second)

## Testing

Comprehensive test suite covering:
- Unit tests for individual components
- Integration tests for search functionality
- Performance benchmarks

Run the tests with:
```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue to discuss potential improvements or report bugs. 