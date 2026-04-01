//! Normalization methods for expression matrices

use crate::{Error, ExpressionMatrix, Result};

/// Normalization methods
pub struct Normalize;

impl Normalize {
    /// Log2 transformation: log2(x+1)
    pub fn log2(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement log2 normalization
        todo!("Implement log2 normalization")
    }

    /// Quantile normalization
    pub fn quantile(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement quantile normalization
        todo!("Implement quantile normalization")
    }

    /// Z-score normalization per gene
    pub fn z_score_per_gene(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        // TODO: Implement z-score normalization
        todo!("Implement z-score normalization")
    }
}
