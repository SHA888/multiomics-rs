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

let reader = SoftReader::open("GSE65682_family.soft.gz")?;
for record in reader.series() {
    let gse = record?;
    println!("{}: {} samples", gse.accession, gse.samples.len());
}

// Get expression matrix as Arrow RecordBatch
let batches = reader.expression_matrix("GSE65682")?;
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
