# File Finder Benchmark Results

## System Information

- **CPU Cores**: 8
- **CPU Model**: Intel(R) Core(TM) i5-10300H CPU @ 2.50GHz
- **RAM (GB)**: 7.8
- **Benchmark Date**: 2025-04-04 11:55:33

## Test Datasets

| Dataset | Total Files | Directory Depth | File Extensions |
|---------|-------------|-----------------|----------------|
| medium_moderate | 132 | 2 | .toml, .json, .rs |
| small_shallow | 42 | 2 | .json, .rs, .toml |
| large_deep | 90 | 5 | .json, .rs, .toml |

## File Distribution by Extension

### medium_moderate

| Extension | File Count |
|-----------|------------|
| .toml | 44 |
| .json | 44 |
| .rs | 44 |

### small_shallow

| Extension | File Count |
|-----------|------------|
| .json | 12 |
| .rs | 15 |
| .toml | 15 |

### large_deep

| Extension | File Count |
|-----------|------------|
| .json | 30 |
| .rs | 30 |
| .toml | 30 |

## Performance Results

### Standard Finder

| Dataset | Time (median) |
|---------|---------------|
| small_shallow | 45 µs |
| medium_moderate | 47 µs |
| large_deep | 84 µs |

### Advanced Finder

| Dataset | Time (median) |
|---------|---------------|
| small_shallow | 691 µs |
| medium_moderate | 1.00 ms |
| large_deep | 1.00 ms |

## Comparative Analysis

| Dataset | Standard Finder | Advanced Finder | Difference |
|---------|-----------------|-----------------|------------|
| small_shallow | 45 µs | 691 µs | 15.1x slower |
| medium_moderate | 47 µs | 1.00 ms | 25.7x slower |
| large_deep | 84 µs | 1.00 ms | 16.4x slower |

## Findings

### Observations

- Advanced finder performance compared to standard finder varies by dataset size and structure
- Directory depth has a significant impact on both finder implementations
- File quantity affects the performance gap between standard and advanced finders

### Conclusions

- For small directories with few nesting levels, standard finder is generally more efficient
- For larger directories with deeper nesting, advanced finder's parallel processing may provide advantages
- Optimization of the advanced finder could focus on reducing overhead for small directory structures
