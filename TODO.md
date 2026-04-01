# multiomics-rs — TODO

> Open source. Apache-2.0 / MIT dual-licensed.
> Each crate versions independently. Breaking changes bump MAJOR per crate.
> Format: `[ ]` open · `[x]` done · `[-]` deferred

---

## VERSIONING

```
v0.x.x   each crate: pre-stable, breaking changes allowed in minor
v1.0.0   per crate: stable public API, MSRV policy documented
```

Publish order tracks Biokhor sprint dependencies:
```
geo-soft-rs         Sprint 3  (MARS, GAinS parsing)
transcriptomic-rs   Sprint 3  (depends on geo-soft-rs)
uniprot-rs          Sprint 4  (hypothesis JSON protein_name)
reactome-rs         Sprint 4  (mechanism synthesis pathways)
open-targets-rs     Sprint 6  (Efferent target ranker)
gtex-rs             Sprint 6  (tissue_specificity scoring)
string-rs           Sprint 6  (off-target prediction)
dgidb-rs            Sprint 6  (modality selector)
```

---

## SPRINT 0 — Workspace bootstrap
> Same pattern as clinical-rs. Gate: CI green on empty workspace.

### [x] S0.1 Repository

- [x] S0.1.1 Create `github.com/SHA888/multiomics-rs` (public)
- [x] S0.1.2 Branch protection: `main` protected, require CI pass
- [x] S0.1.3 `LICENSE-MIT` + `LICENSE-APACHE`
- [x] S0.1.4 `CONTRIBUTING.md` — same conventions as clinical-rs
- [x] S0.1.5 `CODE_OF_CONDUCT.md` — Contributor Covenant v2.1
- [x] S0.1.6 `SECURITY.md` — data correctness bugs = security severity

### [x] S0.2 Workspace manifest (`Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
edition = "2024"
rust-version = "1.84.0"
license = "MIT OR Apache-2.0"
repository = "https://github.com/SHA888/multiomics-rs"
authors = ["Kresna Sucandra"]
categories = ["science", "parser-implementations"]
keywords = ["genomics", "transcriptomics", "arrow", "bioinformatics", "omics"]

[workspace.dependencies]
# Arrow
arrow  = "58"
parquet = "58"

# Parsing
csv        = "1.4"
flate2     = "1"       # gzip decompression for .soft.gz, .tsv.gz
quick-xml  = "0.37"    # UniProt XML
memmap2    = "0.9"

# Parallel
rayon = "1.11"

# Error handling
thiserror = "2"
anyhow    = "1"

# Dev
criterion = { version = "0.8", features = ["html_reports"] }
tempfile  = "3"
insta     = "1"
```

- [x] S0.2.1 Write workspace `Cargo.toml`
- [x] S0.2.2 `rust-toolchain.toml` pinning 1.84.0 stable
- [x] S0.2.3 `rustfmt.toml` — same config as clinical-rs
- [x] S0.2.4 Workspace clippy lints: forbid unsafe, pedantic warn
- [x] S0.2.5 `deny.toml` — same policy as clinical-rs

### [x] S0.3 CI/CD

- [x] S0.3.1 `.github/workflows/ci.yml`
  - trigger: push to `main`, PR to `main`
  - jobs: fmt + clippy + nextest + doc + deny
  - matrix: ubuntu-latest (primary), macos-latest
  - Rust cache via `Swatinem/rust-cache@v2`
- [x] S0.3.2 `.github/workflows/release.yml`
  - trigger: tag `<crate>-v*`
  - `cargo publish -p <crate>`
  - GitHub Release with changelog entry
- [x] S0.3.3 `.github/workflows/audit.yml` — nightly `cargo audit`
- [x] S0.3.4 `cliff.toml` — per-crate changelog
- [x] S0.3.5 `release.toml` — per-crate tag convention

### [x] S0.4 Empty crate scaffolding

For each crate: `Cargo.toml` + `src/lib.rs` stub + `README.md` + `CHANGELOG.md`

- [x] geo-soft-rs scaffold
- [x] transcriptomic-rs scaffold
- [x] open-targets-rs scaffold
- [x] gtex-rs scaffold
- [x] string-rs scaffold
- [x] dgidb-rs scaffold
- [x] uniprot-rs scaffold
- [x] reactome-rs scaffold

- [x] S0.4.1 `cargo check --workspace` passes
- [x] S0.4.2 `cargo fmt --all -- --check` passes
- [x] S0.4.3 `cargo clippy --workspace -- -D warnings` passes
- [x] S0.4.4 CI green on `main`

---

## geo-soft-rs — v0.1.0
> Target: Biokhor Sprint 3 · Parser for NCBI GEO SOFT format → Arrow
> Gap confirmed: no Rust SOFT parser exists on crates.io (April 2026)

### [ ] G1.1 SOFT format parser (state machine)

SOFT structure:
```
^SERIES = GSE65682           → section start
!Series_<key> = <value>      → metadata key-value
!sample_table_begin          → data table start
<tab-separated rows>         → expression data
!sample_table_end            → data table end
^SAMPLE = GSM...             → next section
```

- [ ] G1.1.1 `SoftReader` struct — wraps `BufReader<R: Read>`
  - handles gzip via `flate2::read::GzDecoder` (auto-detect .gz)
  - line-by-line state machine: Idle → InSeries → InPlatform → InSample → InTable
- [ ] G1.1.2 `GseRecord` struct:
  - `accession: String`
  - `title: String`
  - `summary: String`
  - `overall_design: String`
  - `submission_date: NaiveDate`
  - `sample_ids: Vec<String>`
  - `platform_ids: Vec<String>`
  - `metadata: HashMap<String, Vec<String>>` (catch-all for other fields)
- [ ] G1.1.3 `GsmRecord` struct:
  - `accession: String`
  - `title: String`
  - `characteristics: HashMap<String, String>` (disease state, treatment, etc.)
  - `platform_id: String`
  - `data_table: Option<DataTable>` (probe ID + VALUE + optional channels)
- [ ] G1.1.4 `GplRecord` struct:
  - `accession: String`
  - `title: String`
  - `technology: String`
  - `annotation_table: Option<DataTable>` (probe ID → gene symbol, entrez ID)
- [ ] G1.1.5 `DataTable` struct:
  - `columns: Vec<ColumnDescriptor>` (name + description)
  - `rows: Vec<Vec<String>>` (raw values as strings — typed later)
- [ ] G1.1.6 Multi-value field handling (`!key = val` appearing multiple times)
- [ ] G1.1.7 gzip streaming without full decompression into memory

### [ ] G1.2 Arrow output

- [ ] G1.2.1 `GsmRecord::to_record_batch() -> RecordBatch`
  - columns: probe_id (Utf8), value (Float64), optional: channel1, channel2
  - one RecordBatch per GSM record
- [ ] G1.2.2 `GplRecord::annotation_batch() -> RecordBatch`
  - columns: probe_id (Utf8), gene_symbol (Utf8), entrez_id (Utf8), description (Utf8)
- [ ] G1.2.3 `GseRecord::metadata_batch() -> RecordBatch`
  - one row per field, columns: accession, key, value

### [ ] G1.3 `SoftReader` API

- [ ] G1.3.1 `SoftReader::open(path) -> Result<Self>`
- [ ] G1.3.2 `SoftReader::open_gz(path) -> Result<Self>`
- [ ] G1.3.3 `SoftReader::series() -> impl Iterator<Item = Result<GseRecord>>`
- [ ] G1.3.4 `SoftReader::samples() -> impl Iterator<Item = Result<GsmRecord>>`
- [ ] G1.3.5 `SoftReader::platforms() -> impl Iterator<Item = Result<GplRecord>>`
- [ ] G1.3.6 `SoftReader::all() -> SoftRecords { series, samples, platforms }`

### [ ] G1.4 Tests

- [ ] G1.4.1 Synthetic SOFT fixtures in `tests/fixtures/`
  - `minimal_gse.soft` — single GSE, one GPL, two GSMs, data tables
  - `minimal_gse.soft.gz` — gzip version of above
  - `multi_section.soft` — multiple concatenated GSE/GSM/GPL sections
- [ ] G1.4.2 Unit tests:
  - section header parsing: `^SERIES = GSE65682` → accession extracted
  - metadata parsing: multi-value fields accumulated correctly
  - table parsing: column descriptors + row values parsed
  - gzip: same output as uncompressed equivalent
- [ ] G1.4.3 Arrow output tests:
  - `to_record_batch()` schema matches declared schema
  - Float64 columns: "NA" values → null, not parse error
  - Row count matches fixture data
- [ ] G1.4.4 Integration test: parse synthetic file end-to-end,
  assert series accession, sample count, platform annotation row count
- [ ] G1.4.5 Property tests: parser handles empty tables, missing fields,
  arbitrary whitespace without panic

### [ ] G1.5 Documentation + release

- [ ] G1.5.1 All public types and methods have `///` doc comments with examples
- [ ] G1.5.2 Crate `README.md` with minimal usage example
- [ ] G1.5.3 `CHANGELOG.md` entry
- [ ] G1.5.4 `cargo doc --no-deps` builds without warnings
- [ ] G1.5.5 Version `0.0.0` → `0.1.0`, publish to crates.io
- [ ] G1.5.6 Verify crates.io page renders

---

## transcriptomic-rs — v0.1.0
> Target: Biokhor Sprint 3 · Depends on: geo-soft-rs

### [ ] T1.1 Expression matrix assembly

- [ ] T1.1.1 `ExpressionMatrix` struct:
  - `genes: Vec<String>` (gene symbols, rows)
  - `samples: Vec<String>` (GSM accessions, columns)
  - `values: RecordBatch` (Float64 columns — one per sample)
- [ ] T1.1.2 `MatrixBuilder::from_soft(reader: SoftReader) -> ExpressionMatrix`
  - join GSM data tables on probe_id
  - map probe_id → gene_symbol via GPL annotation
  - handle multi-probe-per-gene: mean aggregation (configurable)
  - handle missing values: null (not zero)
- [ ] T1.1.3 `SampleMetadata` struct → RecordBatch:
  - columns: gsm_accession, title, characteristic_key, characteristic_value
  - one row per characteristic per sample
- [ ] T1.1.4 `PlatformAnnotation` struct → RecordBatch:
  - columns: probe_id, gene_symbol, entrez_id, description

### [ ] T1.2 Normalization

- [ ] T1.2.1 `Normalize::log2(matrix) -> ExpressionMatrix` (log2(x+1))
- [ ] T1.2.2 `Normalize::quantile(matrix) -> ExpressionMatrix`
- [ ] T1.2.3 `Normalize::z_score_per_gene(matrix) -> ExpressionMatrix`
- [ ] T1.2.4 Normalization is explicit and composable — no hidden defaults

### [ ] T1.3 Tests + docs + release

- [ ] T1.3.1 Unit tests: matrix assembly from synthetic geo-soft-rs fixtures
- [ ] T1.3.2 Normalization: known-answer tests (specific input → specific output)
- [ ] T1.3.3 Missing value propagation: null in input → null in output
- [ ] T1.3.4 Publish `0.1.0`

---

## uniprot-rs — v0.1.0
> Target: Biokhor Sprint 4

- [ ] U1.1 TSV reader for UniProt Swiss-Prot reviewed human entries
  - columns: Entry, Entry Name, Gene Names, Protein names, Function,
    Subcellular location, GO (all), Reviewed
  - `ProteinAnnotation` → RecordBatch
- [ ] U1.2 REST API fetcher (optional feature `rest-api`)
  - `UniprotClient::fetch_by_gene(symbol) -> Result<ProteinAnnotation>`
  - Used when fresh data preferred over static download
- [ ] U1.3 Tests: known gene symbols return correct protein names
- [ ] U1.4 Publish `0.1.0`

---

## reactome-rs — v0.1.0
> Target: Biokhor Sprint 4

- [ ] R1.1 TSV reader for `Ensembl2Reactome_All_Levels.txt`
  - columns: ensembl_id, reactome_id, url, pathway_name, evidence, species
  - filter: `Homo sapiens` only (configurable)
  - `GenePathway` → RecordBatch
- [ ] R1.2 TSV reader for `ReactomePathways.txt`
  - hierarchy: pathway_id, name, species
  - `PathwayHierarchy` → RecordBatch
- [ ] R1.3 `top_level_pathway(reactome_id) -> String` lookup
- [ ] R1.4 Tests + publish `0.1.0`

---

## open-targets-rs — v0.1.0
> Target: Biokhor Sprint 6

- [ ] O1.1 Parquet reader for Open Targets `evidence/` partition
  - filter by `diseaseId`, `targetId`, `score` threshold
  - `TargetEvidence` → RecordBatch
- [ ] O1.2 Parquet reader for `target/` object
  - `TargetAnnotation` → RecordBatch (tractability, safety flags)
- [ ] O1.3 Parquet reader for `drug/` object
  - `DrugMechanism` → RecordBatch (drug → target → mechanism of action)
- [ ] O1.4 Release version pinning: pin to specific Open Targets release tag
- [ ] O1.5 Tests + publish `0.1.0`

---

## gtex-rs — v0.1.0
> Target: Biokhor Sprint 6

- [ ] X1.1 GCT format reader (tab-delimited, 2-row header, gzip)
  - `GtexReader::from_gct(path) -> Result<Self>`
  - `GtexReader::median_tpm() -> RecordBatch` (gene_id, description, <tissues>)
- [ ] X1.2 Tidy format emitter: pivot wide → long
  - `TissueExpression { gene_id: Utf8, tissue: Utf8, median_tpm: Float64 }`
  - Long format preferred for Arrow columnar efficiency
- [ ] X1.3 Tissue name normalization (GTEx uses verbose names — normalize to short codes)
- [ ] X1.4 Tests + publish `0.1.0`

---

## string-rs — v0.1.0
> Target: Biokhor Sprint 6

- [ ] SR1.1 TSV reader for `9606.protein.links.full.v12.0.txt.gz`
  - space-separated (despite .txt extension)
  - columns: protein1, protein2, combined_score + channel scores
  - `ProteinInteraction` → RecordBatch
- [ ] SR1.2 TSV reader for `9606.protein.info.v12.0.txt.gz`
  - `ProteinInfo { string_id, preferred_name, annotation }` → RecordBatch
- [ ] SR1.3 Directionality: parse v12 regulation direction fields
  - `direction: Option<Direction>` (Activation | Inhibition | Unknown)
- [ ] SR1.4 Default filter: `combined_score >= 700` (configurable)
- [ ] SR1.5 Tests + publish `0.1.0`

---

## dgidb-rs — v0.1.0
> Target: Biokhor Sprint 6
> Note: evaluate DGIdb MCP server before building — may cover runtime needs

- [ ] D1.1 Assess DGIdb MCP server (`github.com/dgidb/dgidb-mcp-server`)
  - If MCP server covers Biokhor's query patterns → use MCP, defer static crate
  - If bulk Arrow output needed → build crate
- [ ] D1.2 (if building) TSV reader for `interactions.tsv`
  - `DrugGeneInteraction` → RecordBatch
- [ ] D1.3 (if building) `genes.tsv` → druggable gene category RecordBatch
- [ ] D1.4 Tests + publish `0.1.0` (if built)

---

## FUTURE — Post v1.0

| Crate | Source | Purpose |
|---|---|---|
| `encode-rs` | ENCODE Project | chromatin accessibility, histone marks → Arrow |
| `tcga-rs` | TCGA via GDC | tumor multi-omics → Arrow |
| `depmap-rs` | DepMap | cancer dependency → Arrow |
| `clinvar-rs` | ClinVar | variant-disease → Arrow |
| `gwas-catalog-rs` | GWAS Catalog | variant-trait → Arrow |
| `bbj-rs` | BioBank Japan | GWAS summary stats → Arrow |

---

*Last updated: workspace created. geo-soft-rs is first priority — fills genuine gap in Rust ecosystem.*
*Immediate next action: S0.1 — create github.com/SHA888/multiomics-rs.*
