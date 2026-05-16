//! Tests for missing value (null) propagation

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
fn test_null_propagates_through_log2() {
    // Matrix with a null value
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["S1", "S2"],
        vec![
            vec![Some(1.0), None],      // GENE_A has null in S2
            vec![Some(3.0), Some(4.0)], // GENE_B all values
        ],
    );

    let normalized = Normalize::log2(&matrix).expect("Failed to apply log2");

    // Null in input should remain null in output
    assert_eq!(
        normalized.get("GENE_A", "S2"),
        None,
        "Null in input should propagate as null through log2"
    );

    // Non-null values should be transformed
    let log2_gene_a_s1 = normalized.get("GENE_A", "S1");
    assert_eq!(log2_gene_a_s1, Some((1.0_f64 + 1.0_f64).log2()));
}

#[test]
fn test_null_propagates_through_z_score() {
    // Matrix with multiple nulls
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![Some(1.0), None, Some(3.0)],   // GENE_A has null in S2
            vec![None, Some(20.0), Some(30.0)], // GENE_B has null in S1
        ],
    );

    let normalized = Normalize::z_score_per_gene(&matrix).expect("Failed to apply z-score");

    // Nulls should propagate
    assert_eq!(
        normalized.get("GENE_A", "S2"),
        None,
        "Null in GENE_A, S2 should propagate through z-score"
    );

    assert_eq!(
        normalized.get("GENE_B", "S1"),
        None,
        "Null in GENE_B, S1 should propagate through z-score"
    );

    // Non-null values should still be computed (using only non-null values for
    // stats)
    let z_a_s1 = normalized.get("GENE_A", "S1");
    assert!(
        z_a_s1.is_some(),
        "Non-null values should be transformed even with some nulls"
    );

    let z_b_s2 = normalized.get("GENE_B", "S2");
    assert!(
        z_b_s2.is_some(),
        "Non-null values should be transformed even with some nulls"
    );
}

#[test]
fn test_null_propagates_through_quantile() {
    // Matrix with nulls
    let matrix = create_test_matrix(
        vec!["GENE_A", "GENE_B"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![Some(1.0), None, Some(5.0)], // GENE_A has null in S2
            vec![Some(3.0), Some(4.0), None], // GENE_B has null in S3
        ],
    );

    let normalized = Normalize::quantile(&matrix).expect("Failed to apply quantile normalization");

    // Nulls should remain null
    assert_eq!(
        normalized.get("GENE_A", "S2"),
        None,
        "Null should propagate through quantile normalization"
    );

    assert_eq!(
        normalized.get("GENE_B", "S3"),
        None,
        "Null should propagate through quantile normalization"
    );

    // Non-null values should still be present
    assert!(
        normalized.get("GENE_A", "S1").is_some(),
        "Non-null values should be transformed"
    );
    assert!(
        normalized.get("GENE_B", "S2").is_some(),
        "Non-null values should be transformed"
    );
}

#[test]
fn test_all_null_gene() {
    // A gene with all null values should remain all null
    let matrix = create_test_matrix(
        vec!["ALL_NULL", "SOME_VALUES"],
        vec!["S1", "S2", "S3"],
        vec![
            vec![None, None, None],                // ALL_NULL
            vec![Some(1.0), Some(2.0), Some(3.0)], // SOME_VALUES
        ],
    );

    let log2 = Normalize::log2(&matrix).expect("Failed log2");
    assert_eq!(log2.get("ALL_NULL", "S1"), None);
    assert_eq!(log2.get("ALL_NULL", "S2"), None);
    assert_eq!(log2.get("ALL_NULL", "S3"), None);

    let zscore = Normalize::z_score_per_gene(&matrix).expect("Failed z-score");
    assert_eq!(zscore.get("ALL_NULL", "S1"), None);
    assert_eq!(zscore.get("ALL_NULL", "S2"), None);
    assert_eq!(zscore.get("ALL_NULL", "S3"), None);

    let quantile = Normalize::quantile(&matrix).expect("Failed quantile");
    assert_eq!(quantile.get("ALL_NULL", "S1"), None);
    assert_eq!(quantile.get("ALL_NULL", "S2"), None);
    assert_eq!(quantile.get("ALL_NULL", "S3"), None);
}

#[test]
fn test_single_value_gene() {
    // A gene with a single non-null value should handle nulls properly
    let matrix = create_test_matrix(
        vec!["MOSTLY_NULL"],
        vec!["S1", "S2", "S3"],
        vec![vec![None, Some(5.0), None]], // Only one value
    );

    // z-score with < 2 values should return original
    let zscore = Normalize::z_score_per_gene(&matrix).expect("Failed z-score");
    assert_eq!(zscore.get("MOSTLY_NULL", "S1"), None);
    assert_eq!(zscore.get("MOSTLY_NULL", "S2"), Some(5.0));
    assert_eq!(zscore.get("MOSTLY_NULL", "S3"), None);

    // log2 should still work
    let log2 = Normalize::log2(&matrix).expect("Failed log2");
    assert_eq!(log2.get("MOSTLY_NULL", "S1"), None);
    assert_eq!(
        log2.get("MOSTLY_NULL", "S2"),
        Some((5.0_f64 + 1.0_f64).log2())
    );
    assert_eq!(log2.get("MOSTLY_NULL", "S3"), None);
}

#[test]
fn test_null_does_not_become_spurious_value() {
    // Null should never accidentally become a real value (0, NaN, etc.)
    let matrix = create_test_matrix(vec!["GENE"], vec!["S1", "S2"], vec![vec![None, Some(10.0)]]);

    for normalize_fn in &[
        ("log2", Normalize::log2(&matrix).expect("log2")),
        (
            "z_score",
            Normalize::z_score_per_gene(&matrix).expect("z_score"),
        ),
        ("quantile", Normalize::quantile(&matrix).expect("quantile")),
    ] {
        let (_name, normalized) = normalize_fn;
        let val = normalized.get("GENE", "S1");
        assert!(val.is_none(), "{}: null should remain null", _name);
        assert!(
            normalized.get("GENE", "S2").is_some(),
            "{}: non-null should remain non-null",
            _name
        );
    }
}
