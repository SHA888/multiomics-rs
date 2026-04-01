//! Error types for geo-soft-rs

use thiserror::Error;

/// Result type for geo-soft-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when parsing SOFT files
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Invalid SOFT format: {0}")]
    InvalidFormat(String),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Gzip error: {0}")]
    Gzip(#[from] flate2::DecompressError),
}
