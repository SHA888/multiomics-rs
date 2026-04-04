//! Target data reader

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for Open Targets target data
pub struct TargetReader {
    // TODO: Implement target reader
}

impl TargetReader {
    /// Create reader from Parquet directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be accessed or is invalid.
    pub fn from_parquet(_path: &str) -> Result<Self> {
        // TODO: Implement Parquet reader
        todo!("Implement target reader")
    }

    /// Read target annotations as `RecordBatches`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
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
