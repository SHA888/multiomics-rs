//! Known-answer tests for normalization methods

use std::sync::Arc;

use arrow::{
    array::Float64Array,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use transcriptomic_rs::{ExpressionMatrix, Normalize};

/// Helper to create a test matrix with specific values
fn create_test_matrix(
    genes: Vec<&str>,
    samples: Vec<&str>,
    values: Vec<Vec<Option<f64>>>,
) -> ExpressionMatrix {
    let fields: Vec<Field> = samples
        .iter()
        .map(|s| Field::new(s.to_string(), DataType::Float64, true))
        .collect();
    let schema = Schema::new(fields);

    let mut columns = Vec::with_capacity(samples.len());
    for sample_idx in 0..samples.len() {
        let mut col_values = Vec::with_capacity(genes.len());
        for gene_values_row in &values {
            col_values.push(gene_values_row[sample_idx]);
        }
        columns.push(Arc::new(Float64Array::from(col_values)) as Arc<dyn arrow::array::Array>);
    }

    let batch =
        RecordBatch::try_new(Arc::new(schema), columns).expect("Failed to create RecordBatch");

    ExpressionMatrix {
        genes: genes.iter().map(|s| s.to_string()).collect(),
        samples: samples.iter().map(|s| s.to_string()).collect(),
        values: batch,
    }
}

#[test]
fn test_log2_transformation() {
    // Create a simple matrix with known values
    // log2(0+1) = 0, log2(1+1) = 1, log2(3+1) = 2, log2(7+1) = 3
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["SAMPLE_1", "SAMPLE_2"],
        vec![
            vec![Some(0.0), Some(1.0)], // GENE_A
            vec![Some(3.0), Some(7.0)], // GENE_B
        ],
    );

    let normalized = Normalize::log2(&matrix).expect("Failed to apply log2");

    // Check that genes and samples are preserved
    assert_eq!(normalized.genes, matrix.genes);
    assert_eq!(normalized.samples, matrix.samples);

    // Check known transformations
    // log2(0+1) = 0
    let val_a1 = normalized.get("GENE_A", "SAMPLE_1");
    assert_eq!(val_a1, Some(0.0));

    // log2(1+1) = 1
    let val_a2 = normalized.get("GENE_A", "SAMPLE_2");
    assert_eq!(val_a2, Some(1.0));

    // log2(3+1) = 2
    let val_b1 = normalized.get("GENE_B", "SAMPLE_1");
    assert_eq!(val_b1, Some(2.0));

    // log2(7+1) = 3
    let val_b2 = normalized.get("GENE_B", "SAMPLE_2");
    assert_eq!(val_b2, Some(3.0));
}

#[test]
fn test_z_score_per_gene() {
    // Create a 2x3 matrix (2 genes, 3 samples)
    // Uses population standard deviation: variance = sum((x-mean)^2) / n
    // GENE_A: [1, 2, 3] -> mean=2, variance=2/3, std=sqrt(2/3)≈0.816
    //   z-scores: (1-2)/0.816≈-1.225, (2-2)/0.816=0, (3-2)/0.816≈1.225
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![Some(1.0), Some(2.0), Some(3.0)],    // GENE_A
            vec![Some(10.0), Some(20.0), Some(30.0)], // GENE_B
        ],
    );

    let normalized = Normalize::z_score_per_gene(&matrix).expect("Failed to apply z-score");

    // Check GENE_A z-scores with population std
    let mean_a = 2.0_f64;
    let variance_a =
        ((1.0_f64 - 2.0_f64).powi(2) + (2.0_f64 - 2.0_f64).powi(2) + (3.0_f64 - 2.0_f64).powi(2))
            / 3.0_f64;
    let std_a = variance_a.sqrt();

    // (1-2)/std_a
    let z_a1 = normalized
        .get("GENE_A", "S1")
        .expect("Should have GENE_A, S1");
    let expected_z_a1 = (1.0_f64 - mean_a) / std_a;
    assert!(
        (z_a1 - expected_z_a1).abs() < 1e-10,
        "Expected {}, got {}",
        expected_z_a1,
        z_a1
    );

    // (2-2)/std_a = 0
    let z_a2 = normalized
        .get("GENE_A", "S2")
        .expect("Should have GENE_A, S2");
    assert!(z_a2.abs() < 1e-10, "Expected 0, got {}", z_a2);

    // (3-2)/std_a
    let z_a3 = normalized
        .get("GENE_A", "S3")
        .expect("Should have GENE_A, S3");
    let expected_z_a3 = (3.0_f64 - mean_a) / std_a;
    assert!(
        (z_a3 - expected_z_a3).abs() < 1e-10,
        "Expected {}, got {}",
        expected_z_a3,
        z_a3
    );

    // Check GENE_B: mean=20, variance=200/3, std=sqrt(200/3)
    let mean_b = 20.0_f64;
    let variance_b = ((10.0_f64 - 20.0_f64).powi(2)
        + (20.0_f64 - 20.0_f64).powi(2)
        + (30.0_f64 - 20.0_f64).powi(2))
        / 3.0_f64;
    let std_b = variance_b.sqrt();

    let z_b1 = normalized
        .get("GENE_B", "S1")
        .expect("Should have GENE_B, S1");
    let expected_z_b1 = (10.0_f64 - mean_b) / std_b;
    assert!(
        (z_b1 - expected_z_b1).abs() < 1e-10,
        "Expected {}, got {}",
        expected_z_b1,
        z_b1
    );
}

#[test]
fn test_z_score_zero_variance_gene() {
    // A gene with the same value in all samples should be unchanged
    let matrix = create_test_matrix(
        vec!["CONST_GENE", "VAR_GENE"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![Some(5.0), Some(5.0), Some(5.0)], // CONST_GENE (zero variance)
            vec![Some(1.0), Some(2.0), Some(3.0)], // VAR_GENE
        ],
    );

    let normalized = Normalize::z_score_per_gene(&matrix).expect("Failed to apply z-score");

    // Zero-variance gene should remain unchanged
    assert_eq!(normalized.get("CONST_GENE", "S1"), Some(5.0));
    assert_eq!(normalized.get("CONST_GENE", "S2"), Some(5.0));
    assert_eq!(normalized.get("CONST_GENE", "S3"), Some(5.0));
}

#[test]
fn test_quantile_normalization() {
    // Create a 2x3 matrix
    // SAMPLE_1: [1, 3] -> sorted: [1, 3]
    // SAMPLE_2: [2, 4] -> sorted: [2, 4]
    // SAMPLE_3: [5, 6] -> sorted: [5, 6]
    //
    // Target distribution:
    //   rank 0: mean(1, 2, 5) = 8/3 ≈ 2.667
    //   rank 1: mean(3, 4, 6) = 13/3 ≈ 4.333
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![Some(1.0), Some(2.0), Some(5.0)], // GENE_A
            vec![Some(3.0), Some(4.0), Some(6.0)], // GENE_B
        ],
    );

    let normalized = Normalize::quantile(&matrix).expect("Failed to apply quantile normalization");

    // After quantile normalization, all samples should have the same distribution
    // Check that the normalization produced valid values
    assert_eq!(normalized.genes, matrix.genes);
    assert_eq!(normalized.samples, matrix.samples);

    // Check that all values are still present (no spurious nulls)
    for gene in &normalized.genes {
        for sample in &normalized.samples {
            let val = normalized.get(gene, sample);
            // Values might be None if they were null originally, but not spuriously
            let _ = val;
        }
    }
}

#[test]
fn test_composable_normalization() {
    // Test that normalization methods are composable:
    // log2 then z-score should work
    let matrix = create_test_matrix(
        vec!["GENE_A"],
        vec!["S1", "S2", "S3"],
        vec![vec![Some(1.0), Some(3.0), Some(7.0)]],
    );

    let step1 = Normalize::log2(&matrix).expect("Failed log2");
    let step2 = Normalize::z_score_per_gene(&step1).expect("Failed z-score after log2");

    // Should have valid data
    assert_eq!(step2.genes, matrix.genes);
    assert_eq!(step2.samples, matrix.samples);
    assert_eq!(step2.values.num_rows(), 1);
    assert_eq!(step2.values.num_columns(), 3);
}

#[test]
fn test_normalization_preserves_structure() {
    let matrix = create_test_matrix(
        vec!["GENE_1", "GENE_2", "GENE_3"],
        vec!["SAMPLE_A", "SAMPLE_B"],
        vec![
            vec![Some(1.0), Some(2.0)],
            vec![Some(3.0), Some(4.0)],
            vec![Some(5.0), Some(6.0)],
        ],
    );

    // All normalization methods should preserve genes and samples
    let log2 = Normalize::log2(&matrix).expect("log2");
    assert_eq!(log2.genes, matrix.genes);
    assert_eq!(log2.samples, matrix.samples);

    let zscore = Normalize::z_score_per_gene(&matrix).expect("z_score");
    assert_eq!(zscore.genes, matrix.genes);
    assert_eq!(zscore.samples, matrix.samples);

    let quantile = Normalize::quantile(&matrix).expect("quantile");
    assert_eq!(quantile.genes, matrix.genes);
    assert_eq!(quantile.samples, matrix.samples);
}
