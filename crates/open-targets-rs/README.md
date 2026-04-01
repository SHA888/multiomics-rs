# open-targets-rs

Open Targets Platform Parquet reader → Arrow RecordBatches.

## Description

`open-targets-rs` provides readers for Open Targets Platform data stored in Parquet format, converting them to Arrow RecordBatches.

## Features

- Evidence data reader with filtering
- Target annotation reader
- Drug mechanism reader
- Arrow-native output

## Quick start

```toml
[dependencies]
open-targets-rs = "0.1"
```

```rust
use open_targets_rs::EvidenceReader;

let reader = EvidenceReader::from_parquet("path/to/evidence/")?;
let batches = reader
    .filter_disease("EFO_0000685")   // sepsis
    .filter_score(0.5)
    .read_batches()?;
```

## License

Licensed under MIT OR Apache-2.0.
