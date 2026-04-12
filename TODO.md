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
memchr     = "2"       # fast byte scanning (\n, \t) for large SOFT files

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

SOFT structure (four entity types, all may appear in a single family file):
```
^PLATFORM = GPL96            → entity start (local ID, not accession)
!Platform_geo_accession = GPL96
!Platform_title = ...
!Platform_table_begin        → annotation table start
ID\tSEQUENCE\tGB_ACC\t...   → header row (tab-delimited)
#ID = probe identifier       → column description (hash line)
<data rows>
!Platform_table_end

^SAMPLE = GSM1234            → entity start (local ID)
!Sample_geo_accession = GSM1234
!Sample_channel_count = 1
!Sample_characteristics_ch1 = disease state: sepsis
!Sample_table_begin
ID_REF\tVALUE\t...
<data rows>
!Sample_table_end

^SERIES = GSE65682
!Series_sample_id = GSM1234
!Series_sample_id = GSM1235

^DATASET = GDS329            → curated dataset (fourth entity type)
!dataset_table_begin
ID_REF\tIDENTIFIER\tGSM1234\tGSM1235\t...
<data rows>
!dataset_table_end

^SUBSET = GDS329_1           → sample grouping within a GDS
!subset_sample_id = GSM1234
```

### [x] G1.0 GDS entity support
> GEO hosts 4,348 curated DataSets in SOFT format — omitting this blocks that corpus

- [x] G1.0.1 `GdsRecord` struct:
  - `geo_accession: String`
  - `title: String`
  - `description: String`
  - `platform: String`
  - `sample_organism: String`
  - `sample_type: String`
  - `feature_count: u32`
  - `sample_count: u32`
  - `subsets: Vec<GdsSubset>`
  - `metadata: HashMap<String, Vec<String>>`
  - `column_descs: HashMap<String, String>`
  - `data_table: Option<DataTable>`
- [x] G1.0.2 `GdsSubset` struct:
  - `local_id: String`
  - `description: String`
  - `sample_ids: Vec<String>`
  - `subset_type: String`
- [x] G1.0.3 State machine transitions for `^DATASET` and `^SUBSET`
- [x] G1.0.4 `!dataset_table_begin` / `!dataset_table_end` handling
- [x] G1.0.5 GDS data table → RecordBatch:
  - columns: `id_ref (Utf8)`, `identifier (Utf8)`, then one `Float64` column per GSM accession
  - column names = GSM accession strings (e.g. `GSM14498`)
  - null sentinel set applies to all Float64 columns
  - sample files [soft](resources/geo)

### [x] G1.1 SOFT format parser (state machine)

State machine — all transitions symmetric; any `^` line while inside an entity emits the
current record and starts the new entity state:
```
Idle → InPlatform     on ^PLATFORM
Idle → InSample       on ^SAMPLE
Idle → InSeries       on ^SERIES
Idle → InDataset      on ^DATASET

InPlatform → InPlatformTable   on !Platform_table_begin
InPlatformTable → InPlatform   on !Platform_table_end
InSample   → InSampleTable     on !Sample_table_begin
InSampleTable   → InSample     on !Sample_table_end
InDataset  → InDatasetTable    on !dataset_table_begin
InDatasetTable  → InDataset    on !dataset_table_end

In any entity state:
  ! line  → attribute accumulation
  # line  → column descriptor accumulation (only valid after header row)
  data line (table state only) → row accumulation
  ^ line  → emit current record, transition to new entity state
  EOF     → emit pending record
```

- [x] G1.1.1 `SoftReader` struct — wraps `BufReader<R: Read>`
  - handles gzip via `flate2::read::GzDecoder` (auto-detect `.gz` suffix)
  - line-by-line state machine per diagram above
- [x] G1.1.2 `GseRecord` struct:
  - `local_id: String` (value from `^SERIES = <this>`)
  - `geo_accession: Option<String>` (from `!Series_geo_accession`)
  - `title: String`
  - `summary: Vec<String>` (multi-value field)
  - `overall_design: String`
  - `series_type: Vec<String>` (download-only; e.g. "Expression profiling by array")
  - `sample_ids: Vec<String>` (from repeated `!Series_sample_id`)
  - `contributor: Vec<String>`
  - `pubmed_id: Vec<u32>`
  - `metadata: HashMap<String, Vec<String>>` (catch-all for non-modeled fields)
- [x] G1.1.3 `GsmRecord` struct:
  - `local_id: String` (value from `^SAMPLE = <this>`)
  - `geo_accession: Option<String>` (from `!Sample_geo_accession`)
  - `title: String`
  - `platform_id: String`
  - `channel_count: u8` (1 or 2; from `!Sample_channel_count` or inferred from `_ch[n]` attributes)
  - `source_name: Vec<String>` (index = channel - 1)
  - `organism: Vec<Vec<String>>` (`[channel][organism]`)
  - `characteristics: Vec<HashMap<String, String>>` (per channel; `Tag: Value` format)
  - `molecule: Vec<String>` (per channel)
  - `label: Vec<String>` (per channel)
  - `data_processing: Vec<String>`
  - `description: Vec<String>`
  - `metadata: HashMap<String, Vec<String>>` (catch-all including download-only fields)
  - `data_table: Option<DataTable>`
- [x] G1.1.4 `GplRecord` struct:
  - `local_id: String` (value from `^PLATFORM = <this>`)
  - `geo_accession: Option<String>` (from `!Platform_geo_accession`)
  - `title: String`
  - `technology: String`
  - `distribution: String`
  - `organism: Vec<String>`
  - `manufacturer: String`
  - `manufacture_protocol: Vec<String>`
  - `description: Vec<String>`
  - `contributor: Vec<String>`
  - `pubmed_id: Vec<u32>`
  - `column_descs: HashMap<String, String>` (from `#` hash lines: col_name → description)
  - `metadata: HashMap<String, Vec<String>>` (catch-all)
  - `annotation_table: Option<DataTable>`
- [x] G1.1.5 `DataTable` struct:
  - `columns: Vec<ColumnDescriptor>` (name + description from `#` hash lines)
  - `rows: Vec<Vec<String>>` (raw strings — typed at Arrow conversion)
- [x] G1.1.6 Multi-value field handling (`!key = val` appearing multiple times → `Vec<String>`)
- [x] G1.1.7 gzip streaming without full decompression into memory
- [x] G1.1.8 Distinguish `^local_id` from `!*_geo_accession` for all entity types —
            these are different values and must not be conflated
- [x] G1.1.9 `channel_count` detection: prefer `!Sample_channel_count`; fall back to
            counting distinct `_ch[n]` suffixes seen in attributes
- [x] G1.1.10 Line ending normalization: strip `\r` before field parsing (`\r\n` and bare `\r`)
- [x] G1.1.11 UTF-8 BOM (`\xEF\xBB\xBF`) stripping on first line of file
- [x] G1.1.12 Blank line tolerance: skip silently in all states
- [x] G1.1.13 Download-only attribute tolerance: `_status`, `_submission_date`,
             `_last_update_date`, `_row_count`, `_contact_*` → route to `metadata` HashMap,
             never return a parse error

### [x] G1.2 Arrow output

- [x] G1.2.1 `GsmRecord::to_record_batch() -> Result<RecordBatch>`
  - single-channel: `id_ref (Utf8)`, `value (Float64, nullable)`
  - dual-channel: `id_ref (Utf8)`, `value (Float64, nullable)` (log ratio),
    `ch1_value (Float64, nullable)`, `ch2_value (Float64, nullable)`
  - auxiliary columns: `Utf8` (caller casts downstream)
  - one RecordBatch per GSM record
- [x] G1.2.2 `GplRecord::annotation_batch() -> Result<RecordBatch>`
  - columns: `id (Utf8)`, `sequence (Utf8)`, `gb_acc (Utf8)`,
    `gene_symbol (Utf8)`, `entrez_id (Utf8)`, `description (Utf8)`
  - standard platform headers (per GEO spec) map to snake_case; non-standard pass through as `Utf8`
  - all columns nullable
- [x] G1.2.3 `GseRecord::metadata_batch() -> Result<RecordBatch>`
  - columns: `accession (Utf8)`, `key (Utf8)`, `value (Utf8)`
  - one row per attribute value; multi-value fields produce multiple rows with same key
- [x] G1.2.4 Null sentinel set — centralized `parse_f64_nullable(s: &str) -> Result<Option<f64>>`:
  - `None` (Arrow null): `""`, `"null"`, `"Null"`, `"NULL"`, `"na"`, `"NA"`,
    `"n/a"`, `"N/A"`, `"nan"`, `"NaN"`, `"NAN"`, `"none"`, `"None"`, `"NONE"`
  - `Ok(Some(f))`: valid float string
  - `Err`: non-null, non-parseable string (surface to caller — do not silently null)
- [x] G1.2.5 Arrow schema metadata on all RecordBatches:
  - `"geo_channel_count"` → `"1"` or `"2"` (GSM batches)
  - `"geo_accession"` → accession string where known
  - `"geo_platform_id"` → platform accession (GSM batches)
- [x] G1.2.6 `#` hash line content propagated into `ColumnDescriptor.description` and
            surfaced as Arrow field metadata (`"geo_col_desc"` key)
- [x] G1.2.7 `GdsRecord::to_record_batch() -> Result<RecordBatch>` (see G1.0.5)

### [x] G1.3 `SoftReader` API

- [x] G1.3.1 `SoftReader::open(path) -> Result<Self>`
- [x] G1.3.2 `SoftReader::open_gz(path) -> Result<Self>`
- [x] G1.3.3 `SoftReader::series() -> impl Iterator<Item = Result<GseRecord>> + '_`
- [x] G1.3.4 `SoftReader::samples() -> impl Iterator<Item = Result<GsmRecord>> + '_`
- [x] G1.3.5 `SoftReader::platforms() -> impl Iterator<Item = Result<GplRecord>> + '_`
- [x] G1.3.6 `SoftReader::datasets() -> impl Iterator<Item = Result<GdsRecord>> + '_`
- [x] G1.3.7 `SoftReader::records() -> impl Iterator<Item = Result<SoftRecord>> + '_`
            — heterogeneous; preserves file order; required for family files where entity
            order is unknown
- [x] G1.3.8 `SoftReader::read_all() -> Result<SoftFile>`
            — eager; convenience for small files

```rust
pub enum SoftRecord {
    Platform(GplRecord),
    Sample(GsmRecord),
    Series(GseRecord),
    Dataset(GdsRecord),
}

pub struct SoftFile {
    pub platforms: Vec<GplRecord>,
    pub samples:   Vec<GsmRecord>,
    pub series:    Vec<GseRecord>,
    pub datasets:  Vec<GdsRecord>,
}
```

### [x] G1.4 Tests

- [x] G1.4.1 Synthetic SOFT fixtures in `tests/fixtures/`
  - `minimal_family.soft` — one GPL, two single-channel GSMs, one GSE, data tables
  - `minimal_family.soft.gz` — gzip version of above
  - `dual_channel.soft` — one GPL, two dual-channel GSMs (VALUE = log ratio)
  - `gds_with_subsets.soft` — one GDS with two `^SUBSET` sections
  - `download_attrs.soft` — file with `_status`, `_contact_*`, `_submission_date` fields
  - `multi_section.soft` — multiple concatenated GSE/GSM/GPL sections
- [x] G1.4.2 Unit tests:
  - entity header parsing: `^SERIES = GSE65682` → `local_id = "GSE65682"`
  - `local_id` vs `geo_accession` differ when fixture has both — verify no conflation
  - metadata parsing: multi-value fields accumulated correctly
  - `#` hash lines: `ColumnDescriptor.description` populated
  - table parsing: column descriptors + row values parsed
  - gzip: same output as uncompressed equivalent
- [x] G1.4.3 Arrow output tests:
  - `to_record_batch()` schema matches declared schema
  - `Float64` columns: all null sentinel strings → Arrow null (not parse error)
  - Row count matches fixture data
  - Schema metadata keys present: `geo_accession`, `geo_channel_count`, `geo_platform_id`
- [x] G1.4.4 Integration test: parse `minimal_family.soft` end-to-end,
  assert series accession, sample count, platform annotation row count
- [x] G1.4.5 Property tests: parser handles empty tables, missing fields,
  arbitrary whitespace without panic
- [x] G1.4.6 Dual-channel fixture: `channel_count = 2`, VALUE column = log ratio,
            `ch1_value` and `ch2_value` columns present in RecordBatch schema
- [x] G1.4.7 GDS fixture: `GdsRecord` parsed, `GdsSubset` list populated,
            data table column count = 2 + sample_count
- [x] G1.4.8 Download-attrs fixture: `_contact_name`, `_status`, `_submission_date`
            route to `metadata` HashMap — no parse error, not in named struct fields
- [x] G1.4.9 Null sentinel coverage: each of `""`, `"NA"`, `"null"`, `"NaN"`, `"none"`
            in VALUE column → Arrow null; verify with `is_null()` on resulting array
- [x] G1.4.10 Malformed float: `"abc"` in VALUE column → `Err`, not `None`
- [x] G1.4.11 Line endings: `\r\n`-terminated fixture produces identical RecordBatch to `\n` version
- [x] G1.4.12 `local_id` vs `geo_accession`: fixture where `^SAMPLE = my_local_name` and
             `!Sample_geo_accession = GSM99999` → `local_id = "my_local_name"`,
             `geo_accession = Some("GSM99999")`
- [x] G1.4.13 Official GDS6063 test with real-world data (7 subsets, 10 samples)

### [x] G1.5 Documentation + release

- [x] G1.5.1 All public types and methods have `///` doc comments with examples
- [x] G1.5.2 Crate `README.md` with minimal usage example
- [x] G1.5.3 `CHANGELOG.md` entry
- [x] G1.5.4 `cargo doc --no-deps` builds without warnings
- [x] G1.5.5 Version `0.0.0` → `0.1.0`
- [ ] G1.5.6 Publish to crates.io and verify page renders

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

*Last updated: geo-soft-rs design review — GDS entity, dual-channel, null sentinels, download attributes, local_id/accession distinction added.*
*Immediate next action: G1.0 + G1.1 implementation.*
