<div align="center">

# multiomics-rs

**Composable Rust crates for multi-omics database ingestion — Arrow as the output contract.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/rust-1.84%2B-orange.svg)](https://www.rust-lang.org)

[Architecture](ARCHITECTURE.md) · [Roadmap](TODO.md) · [Contributing](CONTRIBUTING.md)

</div>

---

## What is this?

`multiomics-rs` is a Cargo workspace of independent crates for ingesting public
multi-omics reference databases into Apache Arrow. Every crate reads a specific
database format and emits Arrow RecordBatches — nothing more.

| Crate | Domain | Omics Area | Source | Format | Status |
|---|---|---|---|---|---|
| [`geo-soft-rs`](crates/geo-soft-rs) | Transcriptomics | Molecular | NCBI GEO | SOFT (line-based text) | 🚧 Pre-release |
| [`transcriptomic-rs`](crates/transcriptomic-rs) | Transcriptomics | Molecular | GEO expression matrices | normalized matrix | 🚧 Pre-release |
| [`open-targets-rs`](crates/open-targets-rs) | Drug target evidence | Translational | Open Targets Platform | Parquet | 🚧 Pre-release |
| [`gtex-rs`](crates/gtex-rs) | Tissue gene expression | Genomics | GTEx v8 | TSV (GCT) | 🚧 Pre-release |
| [`string-rs`](crates/string-rs) | Protein interactions | Proteomics | STRING v12 | TSV | 🚧 Pre-release |
| [`dgidb-rs`](crates/dgidb-rs) | Drug-gene interactions | Pharmacology | DGIdb | TSV | 🚧 Pre-release |
| [`uniprot-rs`](crates/uniprot-rs) | Protein annotation | Proteomics | UniProt Swiss-Prot | TSV / XML | 🚧 Pre-release |
| [`reactome-rs`](crates/reactome-rs) | Biological pathways | Systems biology | Reactome | TSV | 🚧 Pre-release |

## Why Rust?

Multi-omics databases are large. Open Targets is ~20GB of Parquet. STRING human
network is ~1GB TSV. GTEx expression matrix is millions of cells. Python-based
tools impose GIL and memory pressure at this scale. `multiomics-rs` targets that
bottleneck with the same approach as [`clinical-rs`](https://github.com/SHA888/clinical-rs):

- **Arrow-native** — every crate outputs `RecordBatch`. Zero-copy interop with
  PyArrow, Polars, DataFusion, DuckDB.
- **Streaming-first** — `RecordBatchReader` iterators, not materialized collections.
  Same code path for batch (collect → Parquet) and streaming (emit → infer).
- **Parallel where applicable** — `rayon`-based, no GIL.
- **Each crate stands alone** — use `geo-soft-rs` without `open-targets-rs`.
  No forced monolith.

## Scope boundary

`multiomics-rs` handles **reference databases**: transcriptomics, proteomics,
genomics, pharmacology, and pathway data from public repositories.

It does **not** handle:
- Clinical records or patient data → [`clinical-rs`](https://github.com/SHA888/clinical-rs)
- Raw sequencing formats (BAM, VCF, FASTQ) → [`oxbow`](https://github.com/abdenlab/oxbow),
  [`noodles`](https://github.com/zaeleus/noodles)
- Model training or inference
- Any domain-specific application logic

## Quick start

```toml
# Cargo.toml
[dependencies]
geo-soft-rs     = "0.1"   # GEO SOFT → Arrow
transcriptomic-rs = "0.1" # expression matrix normalization → Arrow
open-targets-rs = "0.1"   # Open Targets → Arrow
gtex-rs         = "0.1"   # GTEx tissue expression → Arrow
string-rs       = "0.1"   # STRING protein interactions → Arrow
uniprot-rs      = "0.1"   # UniProt annotation → Arrow
reactome-rs     = "0.1"   # Reactome pathways → Arrow
dgidb-rs        = "0.1"   # DGIdb drug-gene interactions → Arrow
```

### Parse a GEO SOFT file

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

### Load Open Targets evidence

```rust
use open_targets_rs::EvidenceReader;

let reader = EvidenceReader::from_parquet("path/to/evidence/")?;
let batches = reader
    .filter_disease("EFO_0000685")   // sepsis
    .filter_score(0.5)
    .read_batches()?;
```

### Get tissue expression from GTEx

```rust
use gtex_rs::GtexReader;

let reader = GtexReader::from_gct("GTEx_Analysis_v8_gene_median_tpm.gct.gz")?;
let batch = reader.tissue_expression("ENSG00000134045")?;  // IL-6
```

## Design principles

1. **Arrow is the contract.** All crates emit `RecordBatch`. No custom formats.
2. **Each crate stands alone.** `geo-soft-rs` has zero dependency on `string-rs`.
3. **No model training.** Data loading only.
4. **Correctness over cleverness.** Molecular data errors cause scientific harm.
5. **Streaming over materializing.** Handle datasets larger than RAM.

## Relationship to other repositories

```
clinical-rs        clinical records (MIMIC, ICD codes, task windowing)
                   github.com/SHA888/clinical-rs

multiomics-rs      molecular reference databases (this repo)
                   github.com/SHA888/multiomics-rs

oxbow              NGS sequence formats (BAM, VCF, BED, BigWig)
                   github.com/abdenlab/oxbow  [third-party, complementary]

noodles            pure Rust BAM/CRAM/VCF readers
                   github.com/zaeleus/noodles  [third-party, complementary]
```

Neither `clinical-rs` nor `multiomics-rs` depends on the other.
Applications (such as Biokhor) depend on both.

## Requirements

- Rust 1.84+ (2024 edition)
- Data access credentials where required (GEO is open; Open Targets is open;
  GTEx requires dbGaP registration for individual-level data — summary data is open)

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE), at your option.

## Citation

```bibtex
@software{multiomics_rs,
  author  = {Kresna Sucandra},
  title   = {multiomics-rs: Composable Rust crates for multi-omics database ingestion},
  url     = {https://github.com/SHA888/multiomics-rs},
  license = {MIT OR Apache-2.0},
}
```
