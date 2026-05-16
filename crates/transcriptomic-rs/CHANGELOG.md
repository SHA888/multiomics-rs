# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-16

### Added
- Matrix assembly from GEO SOFT format data via `MatrixBuilder`
  - Simple matrix-only assembly with `from_soft(reader)`
  - Complete assembly with sample metadata and platform annotation via `build_all(reader)`
  - Support for multi-probe aggregation: Mean (default), Median, Max, Min
  - Proper gene/sample ordering and null value preservation
- Expression matrix normalization methods
  - Log2 transformation: `Normalize::log2()` for log2(x+1) scaling
  - Z-score per gene: `Normalize::z_score_per_gene()` using population standard deviation
  - Quantile normalization: `Normalize::quantile()` for distribution alignment
  - All methods are composable and return new matrices without modifying originals
- Comprehensive null handling
  - Null values propagate unchanged through all transformations
  - Non-null values computed correctly even in presence of missing data
- Test coverage
  - Matrix assembly tests: fixture loading, dimension validation, aggregation methods
  - Normalization tests: known-answer tests with precise mathematical verification
  - Missing value tests: null propagation across all normalization methods and edge cases
- Documentation
  - Quick start guide with code examples
  - Detailed API documentation for matrix assembly and normalization
  - Properties and composability notes

### Fixed
- Proper handling of genes/samples with zero variance in z-score normalization
- Correct population standard deviation calculation (variance = sum((x-mean)^2) / n)

<!-- next-header -->

<!-- next-url -->
