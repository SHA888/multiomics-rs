# Security Policy

## Supported Versions

Only the latest released version of each crate is supported. Users are encouraged to upgrade to the latest version.

## Reporting a Vulnerability

### Data Correctness Bugs

For multiomics-rs, data correctness bugs are treated with security severity because:

- Scientific research depends on accurate data processing
- Incorrect data transformation can lead to invalid scientific conclusions
- Downstream applications may make critical decisions based on processed data

**Examples of data correctness bugs:**
- Incorrect parsing of genomic coordinates
- Wrong data type conversions (e.g., treating missing values as zeros)
- Incorrect normalization or scaling of expression data
- Mishandling of gene/protein identifiers
- Errors in data aggregation or filtering

### Reporting Process

To report a security vulnerability or data correctness bug:

1. **Do not open a public issue** - send a private report
2. Email: [INSERT SECURITY EMAIL HERE]
3. Include as much detail as possible:
   - Affected crate and version
   - Steps to reproduce the issue
   - Sample data that demonstrates the problem
   - Expected vs actual behavior

### Response Timeline

- **Critical data correctness bugs**: Response within 24 hours, patch within 7 days
- **Other security issues**: Response within 48 hours, patch within 14 days
- **Non-security bugs**: Use regular issue tracker

### Security Best Practices

When working with multiomics data:

- Validate input data formats and constraints
- Handle missing data explicitly (don't assume zeros)
- Use type-safe data representations
- Include comprehensive tests with known-answer fixtures
- Document all data transformations and assumptions
- Consider edge cases (empty datasets, malformed files, etc.)

## Dependencies

We regularly audit and update dependencies. Security updates for dependencies are prioritized and released as patches.

## Disclosure Policy

We follow responsible disclosure practices:

- Security issues are fixed before public disclosure
- Credits are given to reporters in security advisories
- CVEs may be requested for significant issues
- Security advisories are published on GitHub

## Security-Related Features

- All data parsing includes validation
- Memory safety is enforced through Rust's type system
- No unsafe code unless absolutely necessary and audited
- Input sanitization for all external data sources
- Comprehensive test coverage for data processing paths
