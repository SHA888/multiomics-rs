//! Pathway hierarchy data structures

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Reader for Reactome pathway hierarchy data
pub struct PathwayHierarchyReader {
    // TODO: Implement hierarchy reader
}

impl PathwayHierarchyReader {
    /// Create reader from TSV file
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement hierarchy reader")
    }

    /// Read pathway hierarchy as RecordBatch
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
