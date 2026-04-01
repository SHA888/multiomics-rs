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
let matrix = MatrixBuilder::from_soft(reader)?;
let normalized = transcriptomic_rs::Normalize::log2(&matrix);
```

## License

Licensed under MIT OR Apache-2.0.
