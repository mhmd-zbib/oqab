# File Finder

A simple Rust utility that recursively searches for files with a specific extension.

## Features

- Recursively traverses directories starting from a specified root
- Filters files by their extension
- Handles permission errors gracefully
- Returns a list of all matching files
- Uses parallel processing for improved performance on large directories
- Implements multiple design patterns for maintainability and extensibility

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
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── finder.rs        # Core file finding functionality
│   └── cli.rs           # Command-line interface handling
├── .gitignore           # Git ignore configuration
├── Cargo.toml           # Rust package manifest
└── README.md            # This documentation
```

## Architecture and Design Patterns

This project implements several design patterns to ensure clean architecture and separation of concerns:

1. **Strategy Pattern**: Used for file filtering, allowing different filtering strategies to be implemented
2. **Command Pattern**: Encapsulates operations as objects with a common interface
3. **Factory Pattern**: Creates specific finder implementations
4. **Facade Pattern**: Simplifies the interface for finding files
5. **Builder Pattern**: Allows customizing finder configuration

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

### Extensibility

The code is designed to be easily extended:

- To add new file filtering strategies, implement the `FileFilter` trait
- To add new commands, implement the `Command` trait
- All components are loosely coupled for easy modification 