//! Druggable gene data structures

use arrow::record_batch::RecordBatch;

/// Druggable gene data
#[derive(Debug, Clone)]
pub struct DruggableGene {
    pub data: RecordBatch,
}
