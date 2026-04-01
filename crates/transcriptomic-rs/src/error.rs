//! Error types for transcriptomic-rs

use thiserror::Error;

/// Result type for transcriptomic-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during matrix processing
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Matrix error: {0}")]
    Matrix(String),

    #[error("Normalization error: {0}")]
    Normalization(String),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("GEO SOFT error: {0}")]
    GeoSoft(#[from] geo_soft_rs::Error),
}
