## Benchmark Results

### Standard Finder
| Path | File Extension | Time (median) | 
|------|---------------|--------------|
| . (root) | .rs | 463.31 ms |
| . (root) | .toml | 253.52 ms |
| . (root) | .json | 205.69 ms |
| ./src | .rs | 987.76 µs |
| ./src | .toml | 819.70 µs |
| ./src | .json | 857.80 µs |

### Advanced Finder
| Path | File Extension | Time (median) |
|------|---------------|--------------|
| . (root) | .rs | 973.31 ms |
| . (root) | .toml | 901.98 ms |
| . (root) | .json | 1112.10 ms |
| ./src | .rs | 5.54 ms |
| ./src | .toml | 5.05 ms |
| ./src | .json | 4.31 ms |

### Performance Analysis

#### Overall Findings:
- The standard finder is generally faster when searching both the root directory and the src directory
- The advanced finder is significantly slower (5-6x) when searching the src directory
- Both finders perform better in smaller directories as expected

#### Performance by File Type:
- Standard finder shows variability based on file type (.json files found fastest, .rs files slowest)
- Advanced finder maintains more consistent performance across different file types
- Both finders have better performance with .json files in the src directory

#### Optimization Opportunities:
- Advanced finder's implementation could be optimized for smaller directory searches
- Consider using standard finder for simple extension-based searches
- Advanced finder might benefit from path-specific optimizations

These benchmarks were performed on:
- Windows 10
- Intel Core i7 processor
- 16GB RAM
- SSD storage
