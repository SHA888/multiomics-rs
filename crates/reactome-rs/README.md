# reactome-rs

Reactome pathway TSV reader → Arrow RecordBatches.

## Description

`reactome-rs` provides readers for Reactome pathway data, converting them to Arrow RecordBatches.

## Features

- Gene pathway reader
- Pathway hierarchy reader
- Top-level pathway lookup
- Arrow-native output

## Quick start

```toml
[dependencies]
reactome-rs = "0.1"
```

```rust
use reactome_rs::{GenePathwayReader, PathwayHierarchyReader};

let pathways = GenePathwayReader::from_tsv("Ensembl2Reactome_All_Levels.txt")?;
let hierarchy = PathwayHierarchyReader::from_tsv("ReactomePathways.txt")?;
```

## License

Licensed under MIT OR Apache-2.0.
