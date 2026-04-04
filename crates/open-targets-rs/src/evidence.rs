//! Evidence data reader

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for Open Targets evidence data
pub struct EvidenceReader {
    // TODO: Implement evidence reader
}

impl EvidenceReader {
    /// Create reader from Parquet directory
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be accessed or is invalid.
    pub fn from_parquet(_path: &str) -> Result<Self> {
        // TODO: Implement Parquet reader
        todo!("Implement evidence reader")
    }

    /// Filter by disease ID
    pub fn filter_disease(&mut self, _disease_id: &str) -> &mut Self {
        self
    }

    /// Filter by score threshold
    pub fn filter_score(&mut self, _score: f64) -> &mut Self {
        self
    }

    /// Read evidence as `RecordBatches`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or parsed.
    pub fn read_batches(&mut self) -> Result<Vec<RecordBatch>> {
        // TODO: Implement batch reading
        todo!("Implement batch reading")
    }
}

/// Target evidence data
#[derive(Debug, Clone)]
pub struct TargetEvidence {
    pub data: RecordBatch,
}
