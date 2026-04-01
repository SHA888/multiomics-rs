# multiomics-rs Architecture

## Purpose

`multiomics-rs` ingests public multi-omics reference databases and emits Apache
Arrow RecordBatches. It is a data engineering library, not an analysis platform.
Every crate has one job: read a specific format, emit Arrow.

## What it does not do

- No clinical records (→ `clinical-rs`)
- No raw sequencing formats (→ `oxbow`, `noodles`)
- No statistical analysis
- No model training or inference
- No domain-specific application logic (→ consuming applications, e.g. Biokhor)

## Arrow as the universal contract

All crates emit `arrow::record_batch::RecordBatch` via the
`arrow::record_batch::RecordBatchReader` trait. This means:

```
multiomics-rs crate
  reads source format (SOFT, TSV, Parquet, XML)
  emits RecordBatch iterator
    → collect → Parquet (batch use)
    → stream → analysis pipeline (streaming use)
    → PyArrow zero-copy (Python interop)
    → Polars / DataFusion / DuckDB (analytics)
```

No custom serialization formats. No framework lock-in.

## Crate map

```
multiomics-rs workspace
│
├── geo-soft-rs          Layer 1: GEO SOFT format parser
│     GEO SOFT file → GseRecord, GsmRecord, GplRecord
│     One record = one or more RecordBatches
│     Used by: transcriptomic-rs (internal), applications
│
├── transcriptomic-rs    Layer 2: expression matrix normalization
│     Depends on: geo-soft-rs
│     Assembles probe-level GSM records into genes × samples matrix
│     Handles: probe → gene symbol mapping (via GPL annotation)
│     Handles: log2 normalization, batch metadata alignment
│     Emits: ExpressionMatrix, SampleMetadata, PlatformAnnotation
│     Used by: Biokhor Afferent knowledge layer
│
├── open-targets-rs      Drug target evidence → Arrow
│     Reads Open Targets Parquet directly (already columnar)
│     Schema mapping: target, disease, evidence, drug objects
│     Emits: TargetEvidence, DrugMechanism, TargetSafety tables
│     Used by: Biokhor Efferent target ranker
│
├── gtex-rs              Tissue gene expression → Arrow
│     Reads GTEx GCT format (tab-delimited, gzip)
│     Emits: TissueExpression { gene_id, tissue, median_tpm }
│     Used by: Biokhor Efferent tissue_specificity scoring
│
├── string-rs            Protein interaction network → Arrow
│     Reads STRING TSV (compressed)
│     Emits: ProteinInteraction { protein_a, protein_b, score,
│            directionality, channel_scores }
│     Used by: Biokhor Efferent off-target prediction
│
├── dgidb-rs             Drug-gene interactions → Arrow
│     Reads DGIdb TSV
│     Emits: DrugGeneInteraction { gene, drug, interaction_type,
│            pmids, sources, score }
│     Used by: Biokhor Efferent modality selector
│
├── uniprot-rs           Protein annotation → Arrow
│     Reads UniProt Swiss-Prot TSV (reviewed entries only)
│     Emits: ProteinAnnotation { gene_symbol, protein_name,
│            function, subcellular_location, go_terms }
│     Used by: Biokhor Afferent RT engine (protein_name resolution)
│
└── reactome-rs          Biological pathways → Arrow
      Reads Reactome Ensembl2Reactome flat TSV
      Emits: GenePathway { ensembl_id, reactome_id, pathway_name,
             species, top_level_pathway }
      Used by: Biokhor Afferent mechanism synthesis
               Biokhor Efferent modality selector
```

## Layer distinction

```
Layer 1 crates    geo-soft-rs, open-targets-rs, gtex-rs,
                  string-rs, dgidb-rs, uniprot-rs, reactome-rs

  Job:            raw format → structured Arrow records
  Know about:     file format, schema mapping, streaming
  Do not know:    biology, downstream analysis, application logic

Layer 2 crates    transcriptomic-rs

  Job:            structured records → normalized analytical table
  Know about:     expression biology (probe → gene, log normalization)
  Depends on:     geo-soft-rs (layer 1)
  Do not know:    downstream analysis, application logic
```

## Dependency rules

```
Layer 1 crates     no dependency on each other
                   no dependency on clinical-rs
                   no dependency on any application (Biokhor, etc.)

Layer 2 crates     may depend on Layer 1 crates within this workspace
                   no dependency on clinical-rs

Applications       depend on multiomics-rs crates (consumer)
                   multiomics-rs never imports application code
```

## Format notes

### GEO SOFT

```
Structure:
  ^SERIES = GSE65682           section header
  !Series_title = ...          key=value metadata
  !Series_sample_id = GSM...   multi-value fields
  #ID_REF = ...                column descriptor
  !sample_table_begin          data table start marker
  ID_REF  VALUE  ...           tab-separated data
  !sample_table_end            data table end marker

Sections:
  Platform (GPL)  array annotation — probe → gene symbol mapping
  Sample (GSM)    per-sample expression values (probe × 1)
  Series (GSE)    collection of samples + metadata

Compression: gzip (.soft.gz)
Encoding:    UTF-8 line-based
Availability: all GEO records downloadable in SOFT format
Note: GEO discontinued SOFT for submissions in early 2024 but
      continues to serve downloads in SOFT format indefinitely.
```

### Open Targets

```
Format:       Apache Parquet (partitioned by sourceId)
Download:     GCS gs://open-targets-data-releases/
              AWS s3://opentargets-data/
              FTP ftp.ebi.ac.uk/pub/databases/opentargets/
Objects used: evidence/  target/  drug/
Update cycle: quarterly
License:      CC BY 4.0
```

### GTEx v8

```
Format:       GCT (tab-delimited, first two rows are dimension headers)
              Name   Description   <sample_ids...>
              gzip compressed
File:         GTEx_Analysis_2017-06-05_v8_RNASeQCv1.1.9_gene_median_tpm.gct.gz
Dimensions:   56,200 genes × 54 tissue sites (median TPM)
License:      dbGaP open access (summary data, no registration required)
```

### STRING v12

```
Format:       TSV, gzip compressed, space-separated
Organism:     9606 (Homo sapiens)
Files:
  9606.protein.links.full.v12.0.txt.gz  (full with channel scores)
  9606.protein.info.v12.0.txt.gz        (protein metadata)
Filter:       combined_score >= 700 for high-confidence
New in v12:   regulatory directionality (activation/inhibition)
License:      CC BY 4.0
```

### DGIdb

```
Format:       TSV flat files
Files:
  interactions.tsv   drug-gene interactions
  genes.tsv          druggable genome categories
  drugs.tsv          drug metadata, ChEMBL IDs
Update:       monthly releases — re-download at each platform release
License:      MIT (code) + CC BY (data)
MCP server:   github.com/dgidb/dgidb-mcp-server (evaluate vs static download)
```

### UniProt Swiss-Prot

```
Format:       TSV (preferred) — query via REST at build time, or bulk download
              XML (quick-xml crate) — for richer annotation
Scope:        Human reviewed entries only (Swiss-Prot, not TrEMBL)
Size:         ~80MB TSV (human only, ~20,000 proteins)
License:      CC BY 4.0
Strategy:     Query REST API at crate build time for freshness;
              or embed pinned version as build artifact
```

### Reactome

```
Format:       TSV flat file (Ensembl gene → pathway mapping)
Files:
  Ensembl2Reactome_All_Levels.txt
  ReactomePathways.txt
Update:       quarterly
License:      CC BY 4.0
```

## Repository structure

```
multiomics-rs/
├── crates/
│   ├── geo-soft-rs/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs      line-by-line SOFT state machine
│   │   │   ├── record.rs      GseRecord, GsmRecord, GplRecord
│   │   │   └── arrow.rs       record → RecordBatch conversion
│   │   └── Cargo.toml
│   ├── transcriptomic-rs/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── matrix.rs      genes × samples assembly
│   │   │   ├── normalize.rs   log2, quantile normalization
│   │   │   └── metadata.rs    SampleMetadata, PlatformAnnotation
│   │   └── Cargo.toml
│   ├── open-targets-rs/
│   ├── gtex-rs/
│   ├── string-rs/
│   ├── dgidb-rs/
│   ├── uniprot-rs/
│   └── reactome-rs/
├── ARCHITECTURE.md
├── TODO.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── Cargo.toml
```
