//! Protein interaction data structures

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Direction of protein interaction
#[derive(Debug, Clone)]
pub enum Direction {
    Activation,
    Inhibition,
    Unknown,
}

/// Reader for STRING protein interaction data
pub struct ProteinInteractionReader {
    // TODO: Implement interaction reader
}

impl ProteinInteractionReader {
    /// Create reader from TSV file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or parsed.
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement interaction reader")
    }

    /// Read protein interactions as `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
    pub fn read_interactions(&mut self) -> Result<RecordBatch> {
        // TODO: Implement interaction reading
        todo!("Implement interaction reading")
    }
}

/// Protein interaction data
#[derive(Debug, Clone)]
pub struct ProteinInteraction {
    pub data: RecordBatch,
}
