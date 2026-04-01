//! Error types for open-targets-rs

use thiserror::Error;

/// Result type for open-targets-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading Open Targets data
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),
}
