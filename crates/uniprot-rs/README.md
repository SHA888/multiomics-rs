# uniprot-rs

UniProt Swiss-Prot TSV/XML reader → Arrow RecordBatches.

## Description

`uniprot-rs` provides readers for UniProt protein annotation data, converting them to Arrow RecordBatches.

## Features

- TSV reader for Swiss-Prot data
- XML reader (optional feature)
- REST API client (optional feature)
- Arrow-native output

## Quick start

```toml
[dependencies]
uniprot-rs = "0.1"
```

```rust
use uniprot_rs::ProteinReader;

let reader = ProteinReader::from_tsv("uniprot_sprot.tsv")?;
let annotations = reader.read_annotations()?;
```

## License

Licensed under MIT OR Apache-2.0.
