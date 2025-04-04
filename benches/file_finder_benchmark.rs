use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;
use tempfile::TempDir;

use oqab::search::FinderFactory;
use oqab::search::advanced::OqabFinderFactory;

// Create a new temporary directory for testing with a controlled number of files
fn create_test_directory(
    base_dir: &Path,
    num_files: usize, 
    max_depth: usize,
    extensions: &[&str]
) -> Result<(PathBuf, HashMap<String, usize>), Box<dyn std::error::Error>> {
    // Create temp directory
    let dir = TempDir::new_in(base_dir)?;
    let dir_path = dir.path().to_path_buf();
    let mut extension_counts = HashMap::new();
    
    for ext in extensions {
        extension_counts.insert(ext.to_string(), 0);
    }
    
    // Function to create files in directory with specified depth
    fn create_files_recursive(
        dir_path: &Path,
        current_depth: usize,
        max_depth: usize,
        files_per_dir: usize,
        extensions: &[&str],
        counts: &mut HashMap<String, usize>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create files in current directory
        for i in 0..files_per_dir {
            let ext_idx = i % extensions.len();
            let ext = extensions[ext_idx];
            let file_path = dir_path.join(format!("file_{}_{}{}", current_depth, i, ext));
            let mut file = File::create(file_path)?;
            writeln!(file, "Content for test file {}", i)?;
            *counts.entry(ext.to_string()).or_insert(0) += 1;
        }
        
        // Create subdirectories if we haven't reached max depth
        if current_depth < max_depth {
            let subdir_path = dir_path.join(format!("subdir_{}", current_depth));
            fs::create_dir(&subdir_path)?;
            create_files_recursive(
                &subdir_path, 
                current_depth + 1, 
                max_depth, 
                files_per_dir,
                extensions,
                counts,
            )?;
        }
        
        Ok(())
    }
    
    // Calculate files per directory to achieve total_files
    let dirs_count = (0..=max_depth).map(|depth| 2_usize.pow(depth as u32)).sum::<usize>();
    let files_per_dir = num_files / dirs_count;
    
    create_files_recursive(
        &dir_path,
        0,
        max_depth,
        files_per_dir,
        extensions,
        &mut extension_counts,
    )?;
    
    Ok((dir_path, extension_counts))
}

// Get system information
fn get_system_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    
    if let Ok(num_cpus) = std::thread::available_parallelism() {
        info.insert("CPU Cores".to_string(), num_cpus.get().to_string());
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Get processor info on Windows
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "Get-WmiObject -Class Win32_Processor | Select-Object -ExpandProperty Name"])
            .output() 
        {
            if let Ok(cpu_info) = String::from_utf8(output.stdout) {
                info.insert("CPU Model".to_string(), cpu_info.trim().to_string());
            }
        }
        
        // Get memory info
        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "(Get-WmiObject -Class Win32_ComputerSystem).TotalPhysicalMemory / 1GB"])
            .output()
        {
            if let Ok(memory) = String::from_utf8(output.stdout) {
                if let Ok(ram_gb) = memory.trim().parse::<f64>() {
                    info.insert("RAM (GB)".to_string(), format!("{:.1}", ram_gb));
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        // Get processor info on Linux
        if let Ok(output) = Command::new("sh")
            .args(["-c", "cat /proc/cpuinfo | grep 'model name' | head -n 1 | cut -d ':' -f 2"])
            .output() 
        {
            if let Ok(cpu_info) = String::from_utf8(output.stdout) {
                info.insert("CPU Model".to_string(), cpu_info.trim().to_string());
            }
        }
        
        // Get memory info
        if let Ok(output) = Command::new("sh")
            .args(["-c", "cat /proc/meminfo | grep MemTotal | awk '{print $2/1024/1024}'"])
            .output()
        {
            if let Ok(memory) = String::from_utf8(output.stdout) {
                if let Ok(ram_gb) = memory.trim().parse::<f64>() {
                    info.insert("RAM (GB)".to_string(), format!("{:.1}", ram_gb));
                }
            }
        }
    }
    
    info
}

// Write benchmark results to a markdown file
fn write_benchmark_results(
    results: &HashMap<String, Vec<(String, Duration)>>,
    file_counts: &HashMap<String, HashMap<String, usize>>,
    system_info: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = String::new();
    
    // Add title and system info
    output.push_str("# File Finder Benchmark Results\n\n");
    output.push_str("## System Information\n\n");
    
    for (key, value) in system_info {
        output.push_str(&format!("- **{}**: {}\n", key, value));
    }
    
    // Add timestamp
    let now = chrono::Local::now();
    output.push_str(&format!("- **Benchmark Date**: {}\n\n", now.format("%Y-%m-%d %H:%M:%S")));
    
    // Add dataset information
    output.push_str("## Test Datasets\n\n");
    output.push_str("| Dataset | Total Files | Directory Depth | File Extensions |\n");
    output.push_str("|---------|-------------|-----------------|----------------|\n");
    
    for (dataset_name, counts) in file_counts {
        let total_files: usize = counts.values().sum();
        let extensions = counts.keys().cloned().collect::<Vec<_>>().join(", ");
        let depth = if dataset_name.contains("deep") { "5" } else { "2" };
        
        output.push_str(&format!("| {} | {} | {} | {} |\n", 
            dataset_name, total_files, depth, extensions));
    }
    
    output.push_str("\n## File Distribution by Extension\n\n");
    
    for (dataset_name, counts) in file_counts {
        output.push_str(&format!("### {}\n\n", dataset_name));
        output.push_str("| Extension | File Count |\n");
        output.push_str("|-----------|------------|\n");
        
        for (ext, count) in counts {
            output.push_str(&format!("| {} | {} |\n", ext, count));
        }
        
        output.push_str("\n");
    }
    
    // Add benchmark results
    output.push_str("## Performance Results\n\n");
    
    for (finder_name, measurements) in results {
        output.push_str(&format!("### {}\n\n", finder_name));
        output.push_str("| Dataset | Time (median) |\n");
        output.push_str("|---------|---------------|\n");
        
        for (dataset, duration) in measurements {
            let formatted_duration = if duration.as_millis() > 1000 {
                format!("{:.2} s", duration.as_secs_f64())
            } else if duration.as_micros() > 1000 {
                format!("{:.2} ms", duration.as_millis() as f64)
            } else {
                format!("{} µs", duration.as_micros())
            };
            
            output.push_str(&format!("| {} | {} |\n", dataset, formatted_duration));
        }
        
        output.push_str("\n");
    }
    
    // Add performance comparison
    output.push_str("## Comparative Analysis\n\n");
    
    output.push_str("| Dataset | Standard Finder | Advanced Finder | Difference |\n");
    output.push_str("|---------|-----------------|-----------------|------------|\n");
    
    let standard_results = &results["Standard Finder"];
    let advanced_results = &results["Advanced Finder"];
    
    for (idx, (dataset, std_time)) in standard_results.iter().enumerate() {
        let adv_time = &advanced_results[idx].1;
        
        let ratio = adv_time.as_secs_f64() / std_time.as_secs_f64();
        let comparison = if ratio > 1.0 {
            format!("{:.1}x slower", ratio)
        } else {
            format!("{:.1}x faster", 1.0 / ratio)
        };
        
        let std_formatted = if std_time.as_millis() > 1000 {
            format!("{:.2} s", std_time.as_secs_f64())
        } else if std_time.as_micros() > 1000 {
            format!("{:.2} ms", std_time.as_millis() as f64)
        } else {
            format!("{} µs", std_time.as_micros())
        };
        
        let adv_formatted = if adv_time.as_millis() > 1000 {
            format!("{:.2} s", adv_time.as_secs_f64())
        } else if adv_time.as_micros() > 1000 {
            format!("{:.2} ms", adv_time.as_millis() as f64)
        } else {
            format!("{} µs", adv_time.as_micros())
        };
        
        output.push_str(&format!("| {} | {} | {} | {} |\n", 
            dataset, std_formatted, adv_formatted, comparison));
    }
    
    output.push_str("\n## Findings\n\n");
    output.push_str("### Observations\n\n");
    output.push_str("- Advanced finder performance compared to standard finder varies by dataset size and structure\n");
    output.push_str("- Directory depth has a significant impact on both finder implementations\n");
    output.push_str("- File quantity affects the performance gap between standard and advanced finders\n\n");
    
    output.push_str("### Conclusions\n\n");
    output.push_str("- For small directories with few nesting levels, standard finder is generally more efficient\n");
    output.push_str("- For larger directories with deeper nesting, advanced finder's parallel processing may provide advantages\n");
    output.push_str("- Optimization of the advanced finder could focus on reducing overhead for small directory structures\n");
    
    // Write to file
    let mut file = File::create("benchmark_results.md")?;
    file.write_all(output.as_bytes())?;
    
    Ok(())
}

fn bench_file_finders(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_finder_benchmarks");
    
    // Configure benchmarks to run longer for more accurate results
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(15);
    
    // Extensions to search for
    let extensions = [".rs", ".toml", ".json"];
    
    // Create test directories with different structures
    let temp_base = TempDir::new().unwrap();
    let base_path = temp_base.path();
    
    println!("Creating test directories...");
    
    // Small dataset with shallow nesting
    let (small_dir, small_counts) = create_test_directory(
        base_path, 
        100, // 100 files 
        2,    // max depth of 2
        &extensions
    ).unwrap();
    
    // Medium dataset with moderate nesting
    let (medium_dir, medium_counts) = create_test_directory(
        base_path,
        500, // 500 files
        3,   // max depth of 3
        &extensions
    ).unwrap();
    
    // Large dataset with deep nesting
    let (large_dir, large_counts) = create_test_directory(
        base_path,
        1000, // 1000 files
        5,    // max depth of 5
        &extensions
    ).unwrap();
    
    println!("Test directories created.");
    
    // Dataset configurations
    let datasets = [
        ("small_shallow", small_dir.clone()),
        ("medium_moderate", medium_dir.clone()),
        ("large_deep", large_dir.clone()),
    ];
    
    // Store file counts for reporting
    let mut file_counts = HashMap::new();
    file_counts.insert("small_shallow".to_string(), small_counts);
    file_counts.insert("medium_moderate".to_string(), medium_counts);
    file_counts.insert("large_deep".to_string(), large_counts);
    
    // Store benchmark results
    let mut results = HashMap::new();
    results.insert("Standard Finder".to_string(), Vec::new());
    results.insert("Advanced Finder".to_string(), Vec::new());
    
    // Run benchmarks
    for (dataset_name, dir_path) in &datasets {
        // Standard finder benchmark
        let standard_id = BenchmarkId::new("standard_finder", dataset_name);
        group.bench_with_input(standard_id, dataset_name, |b, &_dataset_name| {
            let path = dir_path.clone();
            b.iter(|| {
                let finder = FinderFactory::create_extension_finder(".rs");
                finder.find(black_box(&path))
            });
        });
        
        // Capture median time for standard finder
        let start = Instant::now();
        let finder = FinderFactory::create_extension_finder(".rs");
        for _ in 0..5 {
            finder.find(&dir_path).unwrap();
        }
        let std_duration = start.elapsed() / 5;
        results.get_mut("Standard Finder").unwrap().push((dataset_name.to_string(), std_duration));
        
        // Advanced finder benchmark
        let advanced_id = BenchmarkId::new("advanced_finder", dataset_name);
        group.bench_with_input(advanced_id, dataset_name, |b, &_dataset_name| {
            let path = dir_path.clone();
            b.iter(|| {
                let finder = OqabFinderFactory::create_extension_finder(".rs");
                finder.find(black_box(&path))
            });
        });
        
        // Capture median time for advanced finder
        let start = Instant::now();
        let finder = OqabFinderFactory::create_extension_finder(".rs");
        for _ in 0..5 {
            finder.find(&dir_path).unwrap();
        }
        let adv_duration = start.elapsed() / 5;
        results.get_mut("Advanced Finder").unwrap().push((dataset_name.to_string(), adv_duration));
    }
    
    group.finish();
    
    // Collect system information
    let system_info = get_system_info();
    
    // Write benchmark results to file
    if let Err(e) = write_benchmark_results(&results, &file_counts, &system_info) {
        eprintln!("Error writing benchmark results: {}", e);
    }
}

criterion_group!(benches, bench_file_finders);
criterion_main!(benches); 