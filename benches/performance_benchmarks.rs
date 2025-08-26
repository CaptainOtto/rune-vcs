use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use rune_delta;
use rune_pack;
use rune_store::{Store, Author};
use tempfile::TempDir;
use std::collections::HashMap;

fn benchmark_delta_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_compression");
    
    // Test with different data sizes
    for size in [1024, 10240, 102400].iter() {
        let base = vec![b'A'; *size];
        let mut modified = base.clone();
        // Simulate realistic changes
        modified.extend_from_slice(b" MODIFIED");
        
        group.bench_with_input(
            BenchmarkId::new("compress", size),
            size,
            |b, _| {
                b.iter(|| {
                    rune_delta::make(&base, &modified, 8).unwrap()
                });
            },
        );
    }
    group.finish();
}

fn benchmark_pack_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("pack_operations");
    
    // Create test data
    let mut blobs = Vec::new();
    for i in 0..100 {
        blobs.push((
            format!("file_{}.txt", i),
            format!("Content of file {} with some data", i).into_bytes(),
        ));
    }
    
    group.bench_function("pack_100_files", |b| {
        b.iter(|| {
            rune_pack::pack_blobs(blobs.clone()).unwrap()
        });
    });
    
    group.finish();
}

fn benchmark_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_operations");
    
    group.bench_function("commit_operation", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let store = Store::open(temp_dir.path()).unwrap();
                store.create().unwrap();
                
                // Stage a file
                std::fs::write(store.root.join("test.txt"), "test content").unwrap();
                store.stage_file("test.txt").unwrap();
                
                (temp_dir, store)
            },
            |(_, store)| {
                let author = Author {
                    name: "Benchmark".to_string(),
                    email: "bench@test.com".to_string(),
                };
                store.commit("Benchmark commit", author).unwrap()
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_delta_compression,
    benchmark_pack_operations,
    benchmark_store_operations
);
criterion_main!(benches);
