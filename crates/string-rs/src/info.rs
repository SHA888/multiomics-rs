//! Protein info data structures

use arrow::record_batch::RecordBatch;

/// Protein information data
#[derive(Debug, Clone)]
pub struct ProteinInfo {
    pub data: RecordBatch,
}
