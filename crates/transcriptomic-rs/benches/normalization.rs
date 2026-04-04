//! Benchmark for expression matrix normalization

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_log2_normalization(c: &mut Criterion) {
    // TODO: Add actual benchmark with test data
    c.bench_function("log2_normalization", |b| {
        b.iter(|| {
            // Placeholder for actual benchmark
            black_box(42)
        })
    });
}

fn bench_quantile_normalization(c: &mut Criterion) {
    // TODO: Add actual benchmark with test data
    c.bench_function("quantile_normalization", |b| {
        b.iter(|| {
            // Placeholder for actual benchmark
            black_box(42)
        })
    });
}

criterion_group!(
    benches,
    bench_log2_normalization,
    bench_quantile_normalization
);
criterion_main!(benches);
