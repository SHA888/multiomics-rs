# string-rs

STRING protein interaction TSV reader → Arrow RecordBatches.

## Description

`string-rs` provides readers for STRING protein interaction data, converting them to Arrow RecordBatches.

## Features

- Protein interaction reader with filtering
- Protein information reader
- Directionality parsing
- Arrow-native output

## Quick start

```toml
[dependencies]
string-rs = "0.1"
```

```rust
use string_rs::ProteinInteractionReader;

let reader = ProteinInteractionReader::from_tsv("9606.protein.links.full.v12.0.txt.gz")?;
let interactions = reader.read_interactions()?;
```

## License

Licensed under MIT OR Apache-2.0.
