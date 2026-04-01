//! Expression matrix structures and builders

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Expression matrix with genes as rows and samples as columns
#[derive(Debug, Clone)]
pub struct ExpressionMatrix {
    pub genes: Vec<String>,
    pub samples: Vec<String>,
    pub values: RecordBatch,
}

/// Builder for creating expression matrices from SOFT data
pub struct MatrixBuilder;

impl MatrixBuilder {
    /// Build expression matrix from SOFT reader
    pub fn from_soft(
        _reader: geo_soft_rs::SoftReader<std::io::BufReader<std::fs::File>>,
    ) -> Result<ExpressionMatrix> {
        // TODO: Implement matrix assembly
        todo!("Implement matrix assembly from SOFT data")
    }
}

/// Sample metadata as Arrow RecordBatch
#[derive(Debug, Clone)]
pub struct SampleMetadata {
    pub data: RecordBatch,
}

/// Platform annotation as Arrow RecordBatch
#[derive(Debug, Clone)]
pub struct PlatformAnnotation {
    pub data: RecordBatch,
}
