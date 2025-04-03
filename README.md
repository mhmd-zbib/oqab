# HyperSearch File Finder

A high-performance, feature-rich file finding utility built in Rust.

## Features

- Fast and efficient file searching
- Multiple search modes (standard and advanced)
- Search by file extension or name pattern
- Parallel processing for improved performance
- Progress reporting and silent mode
- Configuration file support
- Comprehensive error handling

## Installation

### From Source

1. Clone the repository
```bash
git clone https://github.com/yourusername/hypersearch.git
cd hypersearch
```

2. Build with Cargo
```bash
cargo build --release
```

3. The binary will be available in `target/release/hypersearch`

## Usage

### Basic Usage

Find all `.rs` files in the current directory:
```bash
hypersearch -e .rs
```

Find files with "config" in their name:
```bash
hypersearch -n config
```

Search in a specific directory:
```bash
hypersearch -p /path/to/search -e .txt
```

Combine extension and name filters:
```bash
hypersearch -e .js -n test
```

### Command Line Options

```
Options:
  -h, --help                Display help information
  -p, --path <PATH>         Directory to search (default: current directory)
  -e, --ext <EXTENSION>     File extension to search for
  -n, --name <NAME>         File name pattern to search for
  -a, --advanced            Use advanced search algorithm
  -s, --silent              Suppress progress output
  -w, --workers <NUMBER>    Number of worker threads (default: number of CPU cores)
  -c, --config <FILE>       Load configuration from file
  --save-config <FILE>      Save current configuration to file
```

### Configuration Files

You can save and load search configurations using JSON files. An example configuration file can be found in the `examples/config.json`:

```json
{
  "path": "src",
  "file_extension": "rs",
  "file_name": "advanced",
  "advanced_search": true,
  "thread_count": 4,
  "show_progress": true,
  "recursive": true,
  "follow_symlinks": false,
  "traversal_strategy": "BreadthFirst"
}
```

Load a configuration:
```bash
hypersearch -c examples/config.json
```

Save current settings to a configuration file:
```bash
hypersearch -e rs -n test --save-config my_search.json
```

### Benchmarking

HyperSearch includes benchmarks to measure and compare the performance of different search implementations:

Run all benchmarks:
```bash
cargo bench
```

This will run performance tests for both the standard file finder and the advanced hyper file finder, allowing you to compare their relative performance.

## Architecture

HyperSearch is built with a focus on maintainability, extensibility, and performance:

- **Command Pattern**: Different search implementations are encapsulated in command objects
- **Strategy Pattern**: Interchangeable filtering strategies
- **Composite Pattern**: Allows combining multiple filters
- **Builder Pattern**: Fluent API for constructing complex objects
- **Observer Pattern**: Progress reporting during search operations

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request 