# dgidb-rs

DGIdb drug-gene interaction TSV reader → Arrow RecordBatches.

## Description

`dgidb-rs` provides readers for DGIdb drug-gene interaction data, converting them to Arrow RecordBatches.

## Features

- Drug-gene interaction reader
- Druggable gene category reader
- Arrow-native output

## Quick start

```toml
[dependencies]
dgidb-rs = "0.1"
```

```rust
use dgidb_rs::InteractionReader;

let reader = InteractionReader::from_tsv("interactions.tsv")?;
let interactions = reader.read_interactions()?;
```

## License

Licensed under MIT OR Apache-2.0.
