//! Tissue expression data structures

use arrow::record_batch::RecordBatch;

/// Tissue expression data in long format
#[derive(Debug, Clone)]
pub struct TissueExpression {
    pub data: RecordBatch,
}

impl TissueExpression {
    /// Create from wide format RecordBatch
    pub fn from_wide(_wide_batch: &RecordBatch) -> Self {
        // TODO: Implement wide to long conversion
        todo!("Implement wide to long conversion")
    }
}
