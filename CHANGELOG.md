# Changelog

## v1.0.0 - 2023-02-08

### Project Restructuring
- Reorganized project structure with dedicated directories for commands, CLI, etc.
- Improved separation of concerns with clear module boundaries
- Added proper error handling with anyhow and thiserror
- Implemented standard Rust project organization practices

### New Features
- Added configuration module with FileSearchConfig struct
- Added observers module with ProgressReporter and SilentObserver
- Implemented Command pattern with HelpCommand, StandardSearchCommand, and AdvancedSearchCommand
- Added CLI argument parsing with clap
- Added help text module with comprehensive documentation
- Added workers option to control thread count
- Added silent mode to suppress progress output

### Infrastructure Improvements
- Added proper error handling with Result type
- Added logging with env_logger and log crates
- Improved dependency management in Cargo.toml
- Added configuration file support with serde for serialization/deserialization

### Bug Fixes
- Fixed help flag conflict with clap
- Fixed module naming to better reflect functionality
- Fixed search path handling with proper defaults
- Added proper validation for search filters

### Documentation
- Added comprehensive README.md with usage examples
- Added inline documentation for public API
- Added CHANGELOG.md to track version history 