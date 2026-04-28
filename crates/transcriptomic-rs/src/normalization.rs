//! Normalization methods for expression matrices
//!
//! All normalization methods are **explicit and composable**.
//! They take a reference to an `ExpressionMatrix` and return a new
//! `ExpressionMatrix` with transformed values. No hidden defaults are applied.

use arrow::{
    array::{Array, Float64Array},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};

use crate::{Error, ExpressionMatrix, Result};

/// Normalization methods
///
/// Each method transforms expression values in a specific way, returning a new
/// `ExpressionMatrix`. Methods are pure functions: they do not modify the input
/// matrix.
pub struct Normalize;

impl Normalize {
    /// Log2 transformation: log2(x+1)
    ///
    /// Applies `log2(x + 1)` to all non-null expression values.
    /// This transformation compresses the dynamic range and handles
    /// zero values gracefully (log2(0+1) = 0).
    ///
    /// # Errors
    ///
    /// Returns an error if Arrow data construction fails.
    pub fn log2(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        let mut columns: Vec<std::sync::Arc<dyn Array>> = Vec::with_capacity(matrix.samples.len());

        for col_idx in 0..matrix.values.num_columns() {
            let col = matrix.values.column(col_idx);
            let array = col
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Normalization("Expected Float64Array".to_string()))?;

            let transformed: Vec<Option<f64>> = (0..array.len())
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        let x = array.value(i);
                        Some((x + 1.0).log2())
                    }
                })
                .collect();

            columns.push(std::sync::Arc::new(Float64Array::from(transformed)));
        }

        let schema = Schema::new(
            matrix
                .samples
                .iter()
                .map(|s| Field::new(s.clone(), DataType::Float64, true))
                .collect::<Vec<_>>(),
        );

        let batch = RecordBatch::try_new(std::sync::Arc::new(schema), columns)?;

        Ok(ExpressionMatrix {
            genes: matrix.genes.clone(),
            samples: matrix.samples.clone(),
            values: batch,
        })
    }

    /// Quantile normalization across samples
    ///
    /// Normalizes the distribution of expression values across all samples
    /// to have the same distribution (the average quantiles across samples).
    /// This ensures that differences in expression are due to biology, not
    /// technical variation.
    ///
    /// # Algorithm
    ///
    /// 1. Sort values within each sample (column) and compute mean ranks
    /// 2. Replace each value with the mean of values at that rank across
    ///    samples
    /// 3. Unsort to restore original gene order
    ///
    /// # Errors
    ///
    /// Returns an error if Arrow data construction fails.
    pub fn quantile(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        let num_genes = matrix.genes.len();
        let num_samples = matrix.samples.len();

        if num_genes == 0 || num_samples == 0 {
            return Ok(matrix.clone());
        }

        // Collect all columns into Vec<Vec<Option<f64>>>
        let mut sample_values: Vec<Vec<Option<f64>>> = Vec::with_capacity(num_samples);
        for col_idx in 0..num_samples {
            let col = matrix.values.column(col_idx);
            let array = col
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Normalization("Expected Float64Array".to_string()))?;

            let values: Vec<Option<f64>> = (0..num_genes)
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .collect();
            sample_values.push(values);
        }

        // For each gene position, collect all non-null values across samples
        // Sort them and compute target distribution (mean at each rank)
        let mut target_distribution: Vec<f64> = Vec::with_capacity(num_genes);

        // Create sorted non-null values per sample
        let mut sorted_per_sample: Vec<Vec<f64>> = Vec::with_capacity(num_samples);
        for values in &sample_values {
            let mut sorted: Vec<f64> = values.iter().flatten().copied().collect();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            sorted_per_sample.push(sorted);
        }

        // Compute max length of sorted arrays
        let max_sorted_len = sorted_per_sample.iter().map(Vec::len).max().unwrap_or(0);

        // For each rank, compute mean across samples that have a value at that rank
        for rank in 0..max_sorted_len {
            let mut sum = 0.0;
            let mut count = 0;
            for sorted in &sorted_per_sample {
                if let Some(&val) = sorted.get(rank) {
                    sum += val;
                    count += 1;
                }
            }
            if count > 0 {
                target_distribution.push(sum / f64::from(count));
            }
        }

        // For each sample, assign quantile-normalized values
        let mut normalized_columns: Vec<std::sync::Arc<dyn Array>> =
            Vec::with_capacity(num_samples);

        for (sample_idx, values) in sample_values.iter().enumerate() {
            let sorted = &sorted_per_sample[sample_idx];

            let normalized: Vec<Option<f64>> = values
                .iter()
                .map(|&opt_val| {
                    if let Some(val) = opt_val {
                        // Find rank of this value in the sorted array
                        if let Ok(pos) = sorted.binary_search_by(|probe| {
                            probe.partial_cmp(&val).unwrap_or(std::cmp::Ordering::Equal)
                        }) {
                            // Handle ties: find middle rank
                            let mut start = pos;
                            let mut end = pos;
                            while start > 0 && sorted.get(start - 1) == Some(&val) {
                                start -= 1;
                            }
                            while end + 1 < sorted.len() && sorted.get(end + 1) == Some(&val) {
                                end += 1;
                            }
                            let rank = usize::midpoint(start, end);
                            target_distribution.get(rank).copied()
                        } else {
                            // Should not happen if value is from the sorted list
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            normalized_columns.push(std::sync::Arc::new(Float64Array::from(normalized)));
        }

        let schema = Schema::new(
            matrix
                .samples
                .iter()
                .map(|s| Field::new(s.clone(), DataType::Float64, true))
                .collect::<Vec<_>>(),
        );

        let batch = RecordBatch::try_new(std::sync::Arc::new(schema), normalized_columns)?;

        Ok(ExpressionMatrix {
            genes: matrix.genes.clone(),
            samples: matrix.samples.clone(),
            values: batch,
        })
    }

    /// Z-score normalization per gene (row-wise)
    ///
    /// For each gene (row), computes: `(x - mean) / std`
    /// where mean and std are calculated across all samples for that gene.
    /// Genes with zero variance (std = 0) are left unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if Arrow data construction fails.
    pub fn z_score_per_gene(matrix: &ExpressionMatrix) -> Result<ExpressionMatrix> {
        let num_genes = matrix.genes.len();
        let num_samples = matrix.samples.len();

        if num_genes == 0 || num_samples == 0 {
            return Ok(matrix.clone());
        }

        // Collect values per gene (row-wise)
        let mut gene_values: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(num_samples); num_genes];

        for col_idx in 0..num_samples {
            let col = matrix.values.column(col_idx);
            let array = col
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Normalization("Expected Float64Array".to_string()))?;

            for (gene_idx, opt_val) in (0..num_genes)
                .map(|i| {
                    if array.is_null(i) {
                        None
                    } else {
                        Some(array.value(i))
                    }
                })
                .enumerate()
            {
                gene_values[gene_idx].push(opt_val);
            }
        }

        // Compute z-scores per gene
        let mut z_score_columns: Vec<Vec<Option<f64>>> =
            vec![Vec::with_capacity(num_genes); num_samples];

        for gene_row in &gene_values {
            let non_null_values: Vec<f64> = gene_row.iter().flatten().copied().collect();

            if non_null_values.len() < 2 {
                // Not enough values for z-score, keep original
                for (col_idx, &orig) in gene_row.iter().enumerate() {
                    z_score_columns[col_idx].push(orig);
                }
                continue;
            }

            #[allow(clippy::cast_precision_loss)]
            let n = non_null_values.len() as f64;
            let mean = non_null_values.iter().sum::<f64>() / n;
            let variance = non_null_values
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / n;
            let std = variance.sqrt();

            if std < f64::EPSILON {
                // Zero variance, keep original values
                for (col_idx, &orig) in gene_row.iter().enumerate() {
                    z_score_columns[col_idx].push(orig);
                }
            } else {
                for (col_idx, &opt_val) in gene_row.iter().enumerate() {
                    let z = opt_val.map(|v| (v - mean) / std);
                    z_score_columns[col_idx].push(z);
                }
            }
        }

        // Build Arrow arrays
        let mut columns: Vec<std::sync::Arc<dyn Array>> = Vec::with_capacity(num_samples);
        for col_values in z_score_columns {
            columns.push(std::sync::Arc::new(Float64Array::from(col_values)));
        }

        let schema = Schema::new(
            matrix
                .samples
                .iter()
                .map(|s| Field::new(s.clone(), DataType::Float64, true))
                .collect::<Vec<_>>(),
        );

        let batch = RecordBatch::try_new(std::sync::Arc::new(schema), columns)?;

        Ok(ExpressionMatrix {
            genes: matrix.genes.clone(),
            samples: matrix.samples.clone(),
            values: batch,
        })
    }
}
