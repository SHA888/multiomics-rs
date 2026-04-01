//! Error types for dgidb-rs

use thiserror::Error;

/// Result type for dgidb-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading DGIdb data
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TSV format error: {0}")]
    TsvFormat(String),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
}
