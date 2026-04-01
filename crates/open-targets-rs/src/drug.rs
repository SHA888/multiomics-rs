//! Drug data reader

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Reader for Open Targets drug data
pub struct DrugReader {
    // TODO: Implement drug reader
}

impl DrugReader {
    /// Create reader from Parquet directory
    pub fn from_parquet(_path: &str) -> Result<Self> {
        // TODO: Implement Parquet reader
        todo!("Implement drug reader")
    }

    /// Read drug mechanisms as RecordBatches
    pub fn read_batches(&mut self) -> Result<Vec<RecordBatch>> {
        // TODO: Implement batch reading
        todo!("Implement batch reading")
    }
}

/// Drug mechanism data
#[derive(Debug, Clone)]
pub struct DrugMechanism {
    pub data: RecordBatch,
}
