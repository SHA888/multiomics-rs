# transcriptomic-rs

Expression matrix assembly and normalization → Arrow RecordBatches.

## Description

`transcriptomic-rs` provides tools for assembling expression matrices from GEO SOFT data and applying common normalization methods.

## Features

- Matrix assembly from SOFT data
- Log2, quantile, and z-score normalization
- Arrow-native output
- Parallel processing with rayon

## Quick start

```toml
[dependencies]
transcriptomic-rs = "0.1"
geo-soft-rs = "0.1"
```

```rust
use transcriptomic_rs::MatrixBuilder;
use geo_soft_rs::SoftReader;

let reader = SoftReader::open("GSE65682_family.soft.gz")?;
let matrix = MatrixBuilder::new().from_soft(reader)?;
let normalized = transcriptomic_rs::Normalize::log2(&matrix)?;
```

## Matrix assembly

Construct an expression matrix from GEO SOFT data:

```rust
// Simple: expression matrix only
let matrix = MatrixBuilder::new().from_soft(reader)?;

// Complete: matrix + sample metadata + platform annotation
let (matrix, metadata, annotation) = MatrixBuilder::new().build_all(reader)?;
```

**Behavior:**
- Joins sample data tables on probe IDs
- Maps probes to genes via platform annotation
- Aggregates multiple probes per gene (mean by default)
- Preserves null values in output

**Aggregation methods:** Mean, Median, Max, Min

## Normalization

All normalization methods are explicit and composable—no hidden defaults.

```rust
// Log2 transformation: log2(x+1)
let log2 = Normalize::log2(&matrix)?;

// Z-score per gene: (x - mean) / std
let zscore = Normalize::z_score_per_gene(&matrix)?;

// Quantile normalization
let quantile = Normalize::quantile(&matrix)?;

// Compose: log2 then z-score
let composed = Normalize::z_score_per_gene(&Normalize::log2(&matrix)?)?;
```

**Properties:**
- Null values propagate unchanged through all transformations
- Methods return new matrices; original is unmodified
- All transformations preserve gene and sample ordering

## License

Licensed under MIT OR Apache-2.0.
