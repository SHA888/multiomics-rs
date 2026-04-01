# gtex-rs

GTEx GCT format reader → Arrow RecordBatches.

## Description

`gtex-rs` provides a reader for GTEx tissue expression data in GCT format, converting them to Arrow RecordBatches.

## Features

- GCT format reader with gzip support
- Tissue name normalization
- Wide to long format conversion
- Arrow-native output

## Quick start

```toml
[dependencies]
gtex-rs = "0.1"
```

```rust
use gtex_rs::GtexReader;

let reader = GtexReader::from_gct("GTEx_Analysis_v8_gene_median_tpm.gct.gz")?;
let batch = reader.median_tpm()?;
```

## License

Licensed under MIT OR Apache-2.0.
