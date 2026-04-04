//! Benchmark for SOFT parsing

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_soft_parsing(c: &mut Criterion) {
    // TODO: Add actual benchmark with test data
    c.bench_function("soft_parser", |b| {
        b.iter(|| {
            // Placeholder for actual benchmark
            black_box(42)
        })
    });
}

criterion_group!(benches, bench_soft_parsing);
criterion_main!(benches);
