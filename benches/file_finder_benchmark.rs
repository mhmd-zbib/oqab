use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use oqab::finder::{FinderFactory, FileFilter, ExtensionFilter};
use oqab::advanced::{HyperFinderFactory, NullObserver};

fn bench_standard_finder(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_finder");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("standard_finder", |b| {
        b.iter(|| {
            let finder = FinderFactory::create_extension_finder(".rs");
            let _ = finder.find(black_box("."));
        })
    });
    
    group.bench_function("hyper_finder", |b| {
        b.iter(|| {
            let finder = HyperFinderFactory::create_extension_finder(".rs");
            let _ = finder.find(black_box("."));
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_standard_finder);
criterion_main!(benches); 