//! Error types for uniprot-rs

use thiserror::Error;

/// Result type for uniprot-rs
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when reading UniProt data
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TSV format error: {0}")]
    TsvFormat(String),

    #[error("XML format error: {0}")]
    XmlFormat(String),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[cfg(feature = "xml")]
    #[error("XML parse error: {0}")]
    XmlParse(#[from] quick_xml::Error),
}
