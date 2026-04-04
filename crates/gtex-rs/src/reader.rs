//! `GTEx` GCT format reader

use arrow::record_batch::RecordBatch;

use crate::Result;

/// Reader for `GTEx` GCT format files
pub struct GtexReader {
    // TODO: Implement GCT reader
}

impl GtexReader {
    /// Create reader from GCT file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be accessed or parsed.
    pub fn from_gct(_path: &str) -> Result<Self> {
        // TODO: Implement GCT reader
        todo!("Implement GCT reader")
    }

    /// Get median TPM values as `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be read or processed.
    pub fn median_tpm(&mut self) -> Result<RecordBatch> {
        // TODO: Implement median TPM reading
        todo!("Implement median TPM reading")
    }
}
