# File Finder

A simple Rust utility that recursively searches for files with a specific extension.

## Features

- Recursively traverses directories starting from a specified root
- Filters files by their extension
- Handles permission errors gracefully
- Returns a list of all matching files
- Uses parallel processing for improved performance on large directories

## Usage

```bash
cargo run <directory_path> <file_extension>
```

Example:

```bash
cargo run /path/to/search .pdf
```

Note: If you don't include the dot in the extension, it will be added automatically (e.g., `pdf` becomes `.pdf`).

## Project Structure

```
.
├── src/
│   └── main.rs          # Main application code
├── .gitignore           # Git ignore configuration
├── Cargo.toml           # Rust package manifest
└── README.md            # This documentation
```

## Development

### Dependencies

This project uses the following dependencies:
- `rayon`: Used for parallel iterators to speed up directory traversal
- `tempfile` (dev-dependency): Used for creating temporary files and directories during testing

### Running Tests

The application includes unit tests that verify its functionality:

```bash
cargo test
```

## Implementation Details

The file finder works by:

1. Accepting a directory path and file extension as input
2. Recursively traversing the directory structure
3. Collecting all files that match the given extension
4. Reporting the results

Error handling is implemented to handle inaccessible directories without stopping the search process.

### Performance Optimization

For improved performance, the application uses parallel processing via the Rayon crate when traversing directories:

- Subdirectories are processed in parallel when there are more than a few
- Small numbers of directories are processed sequentially to avoid parallelization overhead
- File checking is done efficiently with proper locking to prevent race conditions 