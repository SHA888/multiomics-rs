# geo-soft-rs

Parser for NCBI GEO SOFT format → Arrow RecordBatches.

## Description

`geo-soft-rs` provides a streaming parser for NCBI GEO SOFT format files, converting them to Apache Arrow RecordBatches for efficient downstream processing.

## Features

- Streaming parser for large SOFT files
- Automatic gzip decompression
- Arrow-native output for zero-copy interop
- Memory-efficient for multi-gigabyte datasets

## Quick start

```toml
[dependencies]
geo-soft-rs = "0.1"
```

```rust
use geo_soft_rs::SoftReader;

// Open a gzipped SOFT file
let mut reader = SoftReader::open_gz("GSE65682_family.soft.gz")?;

// Iterate over series records
for record in reader.series() {
    let gse = record?;
    println!("{}: {} samples", gse.local_id, gse.sample_ids.len());
}

// Iterate over samples and convert to Arrow RecordBatch
for record in reader.samples() {
    let sample = record?;
    let batch = sample.to_record_batch()?;
    println!("Sample {}: {} probes", sample.title, batch.num_rows());
}
```

## SOFT Format

The SOFT format has the following structure:

```
^SERIES = GSE65682           → section start
!Series_<key> = <value>      → metadata key-value
!sample_table_begin          → data table start
<tab-separated rows>         → expression data
!sample_table_end            → data table end
^SAMPLE = GSM...             → next section
```

## API

- `SoftReader::open(path)` - Open uncompressed SOFT file
- `SoftReader::open_gz(path)` - Open gzipped SOFT file
- `reader.series()` - Iterator over GSE records
- `reader.samples()` - Iterator over GSM records
- `reader.platforms()` - Iterator over GPL records

## License

Licensed under MIT OR Apache-2.0.
