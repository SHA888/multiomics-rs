//! Pathway hierarchy data structures

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for Reactome pathway hierarchy data
pub struct PathwayHierarchyReader {
    // TODO: Implement hierarchy reader
}

impl PathwayHierarchyReader {
    /// Create reader from TSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or parsed.
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement hierarchy reader")
    }

    /// Read pathway hierarchy as `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
    pub fn read_hierarchy(&mut self) -> Result<RecordBatch> {
        // TODO: Implement hierarchy reading
        todo!("Implement hierarchy reading")
    }
}

/// Pathway hierarchy data
#[derive(Debug, Clone)]
pub struct PathwayHierarchy {
    pub data: RecordBatch,
}
