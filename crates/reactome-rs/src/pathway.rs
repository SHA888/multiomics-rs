//! Gene pathway data structures

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for Reactome gene pathway data
pub struct GenePathwayReader {
    // TODO: Implement pathway reader
}

impl GenePathwayReader {
    /// Create reader from TSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or parsed.
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement pathway reader")
    }

    /// Read gene pathways as `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
    pub fn read_pathways(&mut self) -> Result<RecordBatch> {
        // TODO: Implement pathway reading
        todo!("Implement pathway reading")
    }
}

/// Gene pathway data
#[derive(Debug, Clone)]
pub struct GenePathway {
    pub data: RecordBatch,
}

impl GenePathway {
    /// Get top-level pathway for a Reactome ID
    #[must_use]
    pub fn top_level_pathway(&self, _reactome_id: &str) -> Option<String> {
        // TODO: Implement top-level pathway lookup
        todo!("Implement top-level pathway lookup")
    }
}
