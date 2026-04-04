//! Drug-gene interaction data structures

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for `DGIdb` interaction data
pub struct InteractionReader {
    // TODO: Implement interaction reader
}

impl InteractionReader {
    /// Create reader from TSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or parsed.
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement interaction reader")
    }

    /// Read drug-gene interactions as `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
    pub fn read_interactions(&mut self) -> Result<RecordBatch> {
        // TODO: Implement interaction reading
        todo!("Implement interaction reading")
    }
}

/// Drug-gene interaction data
#[derive(Debug, Clone)]
pub struct DrugGeneInteraction {
    pub data: RecordBatch,
}
