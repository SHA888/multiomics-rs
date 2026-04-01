//! Error types for gtex-rs

use thiserror::Error;

/// Result type for gtex-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading GTEx data
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GCT format error: {0}")]
    GctFormat(String),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Gzip error: {0}")]
    Gzip(#[from] flate2::DecompressError),
}
