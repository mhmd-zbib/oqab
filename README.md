# HyperSearch File Finder

A high-performance Rust utility that recursively searches for files with a specific extension using advanced concurrency techniques.

## Features

- Recursively traverses directories starting from a specified root
- Filters files by their extension
- Handles permission errors gracefully
- Returns a list of all matching files
- Uses parallel processing for improved performance on large directories
- Implements multiple design patterns for maintainability and extensibility
- Provides advanced concurrent search with worker pool architecture
- Multiple traversal strategies for different use cases
- Real-time progress reporting
- Result caching for improved performance on repeated searches

## Usage

Basic usage:
```bash
cargo run <directory_path> <file_extension>
```

Advanced usage with options:
```bash
cargo run <directory_path> <file_extension> [options]
```

Available options:
- `--fast` - Use the high-performance concurrent implementation
- `--progress` - Show real-time progress during search
- `--errors` - Show directory access errors
- `--standard` - Use standard directory traversal
- `--git-aware` - Respect .gitignore files (default for --fast)
- `--breadth-first` - Use breadth-first traversal

Example:
```bash
cargo run /path/to/search .pdf --fast --progress
```

Note: If you don't include the dot in the extension, it will be added automatically (e.g., `pdf` becomes `.pdf`).

## Project Structure

```
.
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── finder.rs        # Core file finding functionality
│   ├── cli.rs           # Command-line interface handling
│   ├── advanced.rs      # Advanced concurrent search implementation
│   └── composite.rs     # Composite filter implementation
├── benches/             # Performance benchmarks
├── .gitignore           # Git ignore configuration
├── Cargo.toml           # Rust package manifest
└── README.md            # This documentation
```

## Architecture and Design Patterns

This project implements several design patterns to ensure clean architecture and separation of concerns:

1. **Strategy Pattern**: Used for file filtering and directory traversal, allowing different strategies to be implemented
2. **Command Pattern**: Encapsulates operations as objects with a common interface
3. **Factory Pattern**: Creates specific finder implementations
4. **Facade Pattern**: Simplifies the interface for finding files
5. **Builder Pattern**: Allows customizing finder configuration
6. **Composite Pattern**: Combines multiple filters with AND/OR operations
7. **Observer Pattern**: Provides notifications during the search process
8. **Adapter Pattern**: Standardizes interfaces for different traversal methods
9. **Null Object Pattern**: Provides no-op implementations for optional components
10. **Worker Pool Pattern**: Efficiently distributes work across multiple threads

## Performance Optimizations

The advanced implementation (`HyperFileFinder`) includes several optimizations:

1. **Worker Pool**: Distributes file checking across multiple worker threads
2. **Efficient Traversal**: Uses specialized libraries for directory traversal
3. **Result Caching**: Stores results for repeated searches
4. **Path Deduplication**: Prevents duplicate results through canonicalization
5. **Adaptive Concurrency**: Adjusts worker count based on available CPU cores
6. **Batched Processing**: Minimizes synchronization overhead

## Development

### Dependencies

This project uses the following dependencies:
- `rayon`: Used for parallel iterators
- `tokio`: Asynchronous runtime
- `crossbeam`: Concurrent data structures and channels
- `dashmap`: Thread-safe concurrent HashMap
- `walkdir`: Efficient directory traversal
- `ignore`: Git-aware directory traversal
- `num_cpus`: CPU core detection
- `tempfile` (dev-dependency): Used for creating temporary files during testing
- `criterion` (dev-dependency): Used for benchmarking

### Running Tests

The application includes unit tests that verify its functionality:

```bash
cargo test
```

### Running Benchmarks

Performance benchmarks are available to compare different implementations:

```bash
cargo bench
```

### Extensibility

The code is designed to be easily extended:

- To add new file filtering strategies, implement the `FileFilter` trait
- To add new directory traversal strategies, implement the `DirectoryTraverser` trait
- To add new commands, implement the `Command` trait
- To add custom search progress reporting, implement the `SearchObserver` trait 