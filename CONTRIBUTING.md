# Contributing to multiomics-rs

Thank you for your interest in contributing to multiomics-rs! This document provides guidelines for contributors.

## Development Environment

### Prerequisites

- Rust 1.84.0 or later (2024 edition)
- Git

### Setup

1. Fork the repository
2. Clone your fork locally
3. Create a new branch for your feature: `git checkout -b feature-name`
4. Install dependencies: `cargo build`
5. Run tests: `cargo test`

## Code Style

We follow the same conventions as clinical-rs:

- Use `cargo fmt` for formatting
- Use `cargo clippy` with pedantic lints
- No `unsafe` code unless absolutely necessary
- All public APIs must be documented with `///` doc comments
- Include examples in doc comments where appropriate

## Testing

- All new features must include tests
- Use `cargo test` to run the test suite
- Use `cargo test --workspace` to test all crates
- Integration tests go in `tests/` directories
- Unit tests go in the same module as the code being tested

## Commit Messages

Follow the Conventional Commits specification:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `style:` for code style changes (formatting, etc.)
- `refactor:` for code refactoring
- `test:` for adding or updating tests
- `chore:` for maintenance tasks

Example: `feat(geo-soft-rs): add support for gzip decompression`

## Pull Request Process

1. Update the README.md with details of your changes if applicable
2. Update the CHANGELOG.md following the Keep a Changelog format
3. Ensure all tests pass: `cargo test --workspace`
4. Ensure formatting: `cargo fmt --all -- --check`
5. Ensure clippy: `cargo clippy --workspace -- -D warnings`
6. Submit a pull request to the `main` branch
7. Wait for code review and CI checks to pass

## Release Process

Releases are automated through GitHub Actions:

1. Create a tag following the pattern `<crate>-v<version>` (e.g., `geo-soft-rs-v0.1.0`)
2. Push the tag to trigger the release workflow
3. The workflow will publish to crates.io and create a GitHub release

## Data Correctness

This project handles scientific data where correctness is critical:

- Data parsing bugs are treated with security severity
- All data transformations must be tested with known-answer fixtures
- Include validation for data format constraints
- Document any assumptions about input data formats

## Crate Structure

Each crate in the workspace is independent:

- Each crate has its own `Cargo.toml` with version
- Crates should not depend on each other unless explicitly required
- Shared dependencies are defined in the workspace `Cargo.toml`
- Each crate must have its own `README.md` and `CHANGELOG.md`

## Questions

If you have questions about contributing:

- Open an issue for discussion
- Check existing issues and PRs for similar work
- Refer to the clinical-rs repository for additional patterns and conventions

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
