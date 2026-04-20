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
- No domain-specific application logic (→ consuming applications)
- No license-restricted molecular sources (→ `multiomics-rs-licensed`: DrugBank,
  PhosphoSitePlus, OncoKB, DisGeNet — these require academic or commercial
  license agreements that preclude embedding test data or running CI against
  real data)
- No non-molecular biomedical references (→ `biomedref-rs`: literature-mining
  associations, environmental exposure, food composition)

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
│     Emits: per-sample expression tables, platform annotation,
│            series metadata, dataset tables with subsets
│
├── transcriptomic-rs    Layer 2: expression matrix normalization
│     Depends on: geo-soft-rs
│     Assembles probe-level GSM records into genes × samples matrix
│     Handles: probe → gene symbol mapping (via GPL annotation)
│     Handles: log2 normalization, batch metadata alignment
│     Emits: ExpressionMatrix, SampleMetadata, PlatformAnnotation
│
├── open-targets-rs      Drug target evidence → Arrow
│     Reads Open Targets Parquet directly (already columnar)
│     Schema mapping: target, disease, evidence, drug objects
│     Emits: TargetEvidence, DrugMechanism, TargetSafety tables
│
├── gtex-rs              Tissue gene expression → Arrow
│     Reads GTEx GCT format (tab-delimited, gzip)
│     Emits: TissueExpression { gene_id, tissue, median_tpm }
│
├── string-rs            Protein interaction network → Arrow
│     Reads STRING TSV (compressed)
│     Emits: ProteinInteraction { protein_a, protein_b, score,
│            directionality, channel_scores }
│
├── dgidb-rs             Drug-gene interactions → Arrow
│     Reads DGIdb TSV
│     Emits: DrugGeneInteraction { gene, drug, interaction_type,
│            pmids, sources, score }
│
├── uniprot-rs           Protein annotation → Arrow
│     Reads UniProt Swiss-Prot TSV (reviewed entries only)
│     Emits: ProteinAnnotation { gene_symbol, protein_name,
│            function, subcellular_location, go_terms }
│
└── reactome-rs          Biological pathways → Arrow
      Reads Reactome Ensembl2Reactome flat TSV
      Emits: GenePathway { ensembl_id, reactome_id, pathway_name,
             species, top_level_pathway }
```

### Crate map — v0.3 additions

```
hgnc-rs                  Layer 1: gene symbol authority → Arrow  [Wave 1]
      Reads HGNC complete dataset (TSV or JSON)
      Emits: GeneSymbol { hgnc_id, approved_symbol, approved_name,
             locus_group, previous_symbols, alias_symbols,
             entrez_id, ensembl_id, refseq_id, uniprot_id }
      Role: identifier authority; other crates use its output as the
      canonical gene-symbol table for ID normalization.

refseq-rs                Layer 1: transcript & protein refs → Arrow  [Wave 2]
      Reads NCBI RefSeq release files (TSV)
      Emits: TranscriptRef { refseq_id, gene_symbol, transcript_type,
             length, chrom, start, end, strand, xrefs }

pfam-rs                  Layer 1: protein families & domains → Arrow  [Wave 2]
      Reads Pfam (now InterPro) TSV / JSON dumps
      Emits: ProteinFamily { pfam_id, pfam_name, description,
             clan_id, member_count }
             DomainHit { uniprot_id, pfam_id, start, end, e_value }

intact-rs                Layer 1: curated interactions → Arrow  [Wave 2]
      Reads PSI-MITAB 2.7 files (IntAct / MINT consortium format)
      Emits: Interaction { id_a, id_b, method, type, publication,
             confidence, source_db }
      PSI-MITAB is the PSI-MI consortium's tab-delimited exchange
      format; IntAct, MINT, and several others all distribute in this
      format so one parser covers multiple sources.

corum-rs                 Layer 1: protein complexes → Arrow  [Wave 2]
      Reads CORUM TSV / XML dumps
      Emits: ProteinComplex { complex_id, complex_name, organism,
             subunits, function_annotation, pubmed_ids,
             disease_associations }

signor-rs                Layer 1: signed signaling networks → Arrow  [Wave 2]
      Reads SIGNOR TSV dumps
      Emits: SignalingEdge { entity_a, entity_b, effect, mechanism,
             residue, sequence, publication, tissue, cell_line }
      Complements STRING by providing curated directionality and
      signed effects (activation vs. inhibition) at higher confidence
      than inferred v12 channel scores.

hmdb-rs                  Layer 1: human metabolome → Arrow  [Wave 3]
      Reads HMDB XML / TSV dumps
      Emits: Metabolite { hmdb_id, name, chemical_formula, avg_mass,
             kegg_id, chebi_id, pubchem_id, smiles, inchi,
             biospecimen_locations, tissue_locations, pathways }

gwas-catalog-rs          Layer 1: variant-trait associations → Arrow  [Wave 3]
      Reads NHGRI-EBI GWAS Catalog TSV
      Emits: GwasAssociation { study_accession, snp_id, trait,
             p_value, odds_ratio, beta, risk_allele,
             mapped_gene, sample_size, ancestry }

hpa-rs                   Layer 1: tissue / cell-type expression → Arrow  [Wave 3]
      Reads Human Protein Atlas download TSV files
      Emits: TissueExpression { gene, tissue, level, reliability }
             SubcellularLocation { gene, location, reliability }
      Overlaps GTEx in tissue scope but adds protein-level IHC
      evidence and subcellular localization data that GTEx lacks.

sider-rs                 Layer 1: drug side-effect mappings → Arrow  [Wave 3]
      Reads SIDER TSV dumps
      Emits: DrugSideEffect { stitch_id, umls_concept, side_effect,
             frequency, placebo_ratio }
             DrugIndication { stitch_id, umls_concept, indication }

stitch-rs                Layer 1: chemical-protein interactions → Arrow  [Wave 3]
      Reads STITCH TSV dumps (structurally similar to STRING)
      Emits: ChemicalProteinInteraction { chemical_id, protein_id,
             combined_score, channel_scores }
      Uses the same tab-separated structure as STRING v12; internal
      parser may share common code path with string-rs (evaluate at
      implementation time).

cgi-rs                   Layer 1: cancer variant interpretation → Arrow  [Wave 3]
      Reads Cancer Genome Interpreter TSV dumps
      Emits: ClinicallyRelevantVariant { gene, variant, disease,
             drug, evidence_level, source_publication }
      License: CC BY-NC-SA 4.0 (non-commercial share-alike).
      Downstream users must comply with the data license; this
      crate surfaces license info in Arrow schema metadata.

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
                   no dependency on any consuming application

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

### HGNC  [v0.3]

```
Format:       TSV (canonical download) or JSON (via REST API)
Download:     ftp.ebi.ac.uk/pub/databases/genenames/hgnc/
              rest.genenames.org/fetch/symbol/<symbol>
File:         hgnc_complete_set.txt
Update:       monthly (nightly internal; monthly stable dumps)
License:      CC0 1.0 Universal (public domain dedication)
Scope:        ~43,000 approved human gene symbols plus withdrawn
              and placeholder records; cross-refs to Entrez, Ensembl,
              RefSeq, UniProt, OMIM, and COSMIC.
```

### RefSeq  [v0.3]

```
Format:       TSV (release files) and GenBank (annotations — out of scope
              for multiomics-rs; see oxbow/noodles for sequence data)
Download:     ftp.ncbi.nlm.nih.gov/refseq/release/
Scope:        Transcript and protein reference sequences; this crate
              parses the release's summary TSV and cross-reference
              tables, not the underlying FASTA/GenBank sequence files
License:      public domain (NCBI data)
```

### Pfam / InterPro  [v0.3]

```
Format:       TSV dumps and JSON API
Status:       Pfam flat files are now distributed as part of InterPro
              (https://www.ebi.ac.uk/interpro/download/pfam/)
Files:        pfamA.txt.gz, pfamA_reg_full_significant.txt.gz
Update:       quarterly (aligned with InterPro releases)
License:      CC0 (public domain)
```

### IntAct (PSI-MITAB 2.7)  [v0.3]

```
Format:       PSI-MITAB 2.7 (tab-delimited, PSI-MI consortium standard)
Download:     ftp.ebi.ac.uk/pub/databases/intact/current/psimitab/
Files:        intact.txt (full) and intact-micluster.txt (clustered)
Scope:        ~1.2M binary interactions; PSI-MITAB is shared by IntAct,
              MINT, and several other PSI-MI consortium members — one
              parser covers all
Update:       monthly
License:      CC BY 4.0
Reference:    Orchard S. et al., Nucleic Acids Research (annual update)
```

### CORUM  [v0.3]

```
Format:       TSV and XML (PSI-MI 2.5)
Download:     mips.helmholtz-muenchen.de/corum/download
Scope:        CORUM 5.0 — ~7,193 manually curated mammalian protein
              complexes (~5,300 genes, 26% of human protein-coding
              genes)
Update:       roughly every 2 years (releases 1.0 → 5.0)
License:      CC BY 4.0 (verified from CORUM 4.0 / 2022 publication)
Reference:    Tsitsiridis G. et al., Nucleic Acids Research 2023
```

### SIGNOR  [v0.3]

```
Format:       TSV (custom schema)
Download:     signor.uniroma2.it/downloads.php
Scope:        Directed, signed causal interactions (activation,
              inhibition, phosphorylation, binding, etc.) with
              residue-level detail where known
Update:       monthly
License:      CC BY-SA 4.0 (share-alike — flag in Arrow metadata)
```

### HMDB  [v0.3]

```
Format:       XML (primary, full records) and TSV (summary tables)
Download:     hmdb.ca/downloads
Scope:        ~220,000 metabolite entries with chemical properties,
              biological roles, tissue/biospecimen localization,
              disease associations, pathway memberships
Update:       roughly annual (major versions) with incremental patches
License:      HMDB custom license — free for academic / non-commercial
              use, contact for commercial use
Note:         XML parsing recommended for completeness; TSV subsets
              omit several nested fields (pathways, references).
              Use quick-xml crate.
```

### NHGRI-EBI GWAS Catalog  [v0.3]

```
Format:       TSV (downloadable full catalog)
Download:     ebi.ac.uk/gwas/docs/file-downloads
Files:        gwas_catalog_v1.0.tsv (main full file with all associations)
Scope:        All published human GWAS with p < 1e-5 associations;
              ~600,000 associations across ~6,000 traits
Update:       weekly
License:      CC0 1.0 Universal (public domain)
```

### Human Protein Atlas  [v0.3]

```
Format:       TSV download files (tissue, cell line, pathology,
              subcellular localization each as separate TSVs)
Download:     proteinatlas.org/about/download
Files:        proteinatlas.tsv.zip (normal tissue IHC)
              pathology.tsv.zip (cancer)
              subcellular_location.tsv.zip
Scope:        Tissue-level protein expression, IHC staining scores,
              subcellular localization, pathology staining
License:      CC BY-SA 4.0 (share-alike — flag in Arrow metadata)
Note:         Complements GTEx by adding protein-level IHC evidence;
              GTEx is RNA-level only.
```

### SIDER  [v0.3]

```
Format:       TSV dumps (version 4.1 is current)
Download:     sideeffects.embl.de/download/
Files:        meddra_all_se.tsv (side effects)
              meddra_all_indications.tsv (indications)
              drug_names.tsv, drug_atc.tsv (drug identifiers)
Scope:        ~1,400 marketed drugs with ~5,880 side effects; maps
              drug names to ATC and STITCH IDs, side effects to MedDRA
              concepts, phenotypes to UMLS CUIs
License:      CC BY-NC-SA 4.0 (non-commercial share-alike —
              flag in Arrow metadata, warn consumers)
Note:         Despite the SIDER website's age, no better freely-
              licensed alternative exists for systematic drug
              side-effect tabulations.
```

### STITCH  [v0.3]

```
Format:       TSV dumps (structurally mirrors STRING format)
Download:     stitch.embl.de/download.html
Files:        9606.protein_chemical.links.v5.0.tsv.gz
              9606.actions.v5.0.tsv.gz (directed interactions)
Scope:        Chemical-protein interactions; chemical IDs are CID-m
              (merged PubChem CIDs) for stereoisomer grouping
License:      CC BY 4.0
Note:         Internal parser may share code with string-rs given
              near-identical file structure; evaluate at implementation.
```

### Cancer Genome Interpreter (CGI)  [v0.3]

```
Format:       TSV (compiled clinical variant catalog)
Download:     cancergenomeinterpreter.org/data
Scope:        Biomarker-based cancer variant annotations and drug
              response predictions; smaller and more curated than
              OncoKB; covers validated and pre-clinical evidence
License:      CC BY-NC-SA 4.0 (non-commercial share-alike —
              flag in Arrow metadata, warn consumers)
Note:         Open access but non-commercial terms restrict downstream
              commercial products. Users must comply with license.
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
│   ├── reactome-rs/
│   ├── hgnc-rs/               # planned, v0.3, Wave 1
│   ├── refseq-rs/             # planned, v0.3, Wave 2
│   ├── pfam-rs/               # planned, v0.3, Wave 2
│   ├── intact-rs/             # planned, v0.3, Wave 2
│   ├── corum-rs/              # planned, v0.3, Wave 2
│   ├── signor-rs/             # planned, v0.3, Wave 2
│   ├── hmdb-rs/               # planned, v0.3, Wave 3
│   ├── gwas-catalog-rs/       # planned, v0.3, Wave 3
│   ├── hpa-rs/                # planned, v0.3, Wave 3
│   ├── sider-rs/              # planned, v0.3, Wave 3
│   ├── stitch-rs/             # planned, v0.3, Wave 3
│   └── cgi-rs/                # planned, v0.3, Wave 3
├── ARCHITECTURE.md
├── TODO.md
├── CONTRIBUTING.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── Cargo.toml
```
