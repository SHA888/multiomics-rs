//! Normalization methods for expression matrices

use crate::{ExpressionMatrix, Result};

/// Normalization methods
pub struct Normalize;

impl Normalize {
    /// Log2 transformation: log2(x+1)
    ///
    /// # Errors
    ///
    /// Returns an error if the normalization fails.
    pub fn log2(_matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement log2 normalization
        todo!("Implement log2 normalization")
    }

    /// Quantile normalization
    ///
    /// # Errors
    ///
    /// Returns an error if the normalization fails.
    pub fn quantile(_matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement quantile normalization
        todo!("Implement quantile normalization")
    }

    /// Z-score normalization per gene
    ///
    /// # Errors
    ///
    /// Returns an error if the normalization fails.
    pub fn z_score_per_gene(_matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement z-score normalization
        todo!("Implement z-score normalization")
    }
}
