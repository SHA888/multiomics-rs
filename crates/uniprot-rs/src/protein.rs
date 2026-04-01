//! Protein annotation data structures

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Reader for UniProt protein data
pub struct ProteinReader {
    // TODO: Implement protein reader
}

impl ProteinReader {
    /// Create reader from TSV file
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement protein reader")
    }

    /// Read protein annotations as RecordBatch
    pub fn read_annotations(&mut self) -> Result<RecordBatch> {
        // TODO: Implement annotation reading
        todo!("Implement annotation reading")
    }

    /// Create reader from XML file
    #[cfg(feature = "xml")]
    pub fn from_xml(_path: &str) -> Result<Self> {
        // TODO: Implement XML reader
        todo!("Implement XML reader")
    }
}

/// Protein annotation data
#[derive(Debug, Clone)]
pub struct ProteinAnnotation {
    pub data: RecordBatch,
}
