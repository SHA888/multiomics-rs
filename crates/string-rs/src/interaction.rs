//! Protein interaction data structures

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

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
    pub fn from_tsv(_path: &str) -> Result<Self> {
        // TODO: Implement TSV reader
        todo!("Implement interaction reader")
    }

    /// Read protein interactions as RecordBatch
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
