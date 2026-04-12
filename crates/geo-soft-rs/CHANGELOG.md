# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-12

### Added

- Initial release of geo-soft-rs
- Streaming parser for NCBI GEO SOFT format files
- Support for all GEO entity types: Series (GSE), Samples (GSM), Platforms (GPL), Datasets (GDS)
- Gzip decompression support via `SoftReader::open_gz()`
- Arrow `RecordBatch` conversion for data tables via `to_record_batch()`
- Heterogeneous iterator via `SoftRecord` enum for mixed file parsing
- Metadata extraction to HashMap for extensibility
- Null sentinel handling ("NA", "null", "NaN", "none", "")
- Column descriptor extraction from `#` comment lines
- Dual-channel (two-color) array support
- Comprehensive test suite with synthetic fixtures and official GDS6063

<!-- next-header -->

[0.1.0]: https://github.com/SHA888/multiomics-rs/releases/tag/geo-soft-rs-v0.1.0
