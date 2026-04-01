//! Target data reader

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Reader for Open Targets target data
pub struct TargetReader {
    // TODO: Implement target reader
}

impl TargetReader {
    /// Create reader from Parquet directory
    pub fn from_parquet(_path: &str) -> Result<Self> {
        // TODO: Implement Parquet reader
        todo!("Implement target reader")
    }

    /// Read target annotations as RecordBatches
    pub fn read_batches(&mut self) -> Result<Vec<RecordBatch>> {
        // TODO: Implement batch reading
        todo!("Implement batch reading")
    }
}

/// Target annotation data
#[derive(Debug, Clone)]
pub struct TargetAnnotation {
    pub data: RecordBatch,
}
