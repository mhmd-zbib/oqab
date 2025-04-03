use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::Path;
use std::time::Duration;

use oqab::search::FinderFactory;
use oqab::search::advanced::{OqabFinderFactory, NullObserver};

fn bench_file_finders(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_finder_benchmarks");
    
    // Configure benchmarks to run longer for more accurate results
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20);
    
    // Benchmark paths to test with varying sizes
    let paths = vec![
        ("small", "."),
        ("src_only", "./src"),
    ];
    
    // Extensions to search for
    let extensions = vec![".rs", ".toml", ".json"];
    
    // Benchmark standard finder
    for path_pair in &paths {
        for ext in &extensions {
            group.bench_with_input(
                BenchmarkId::new("standard_finder", format!("{}/{}", path_pair.0, ext)), 
                &(path_pair.1, ext), 
                |b, (path, ext)| {
                    b.iter(|| {
                        let finder = FinderFactory::create_extension_finder(ext);
                        finder.find(black_box(Path::new(path)))
                    })
                }
            );
        }
    }
    
    // Benchmark advanced finder
    for path_pair in &paths {
        for ext in &extensions {
            group.bench_with_input(
                BenchmarkId::new("advanced_finder", format!("{}/{}", path_pair.0, ext)),
                &(path_pair.1, ext),
                |b, (path, ext)| {
                    b.iter(|| {
                        let observer = Box::new(NullObserver);
                        let finder = OqabFinderFactory::create_extension_finder(ext, observer);
                        finder.find(black_box(Path::new(path)))
                    })
                }
            );
        }
    }
    
    group.finish();
}

criterion_group!(benches, bench_file_finders);
criterion_main!(benches); 