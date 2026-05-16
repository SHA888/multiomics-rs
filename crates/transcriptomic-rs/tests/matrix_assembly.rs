//! Integration tests for matrix assembly from SOFT fixtures

use geo_soft_rs::SoftReader;
use transcriptomic_rs::{AggregationMethod, MatrixBuilder, MatrixConfig};

/// Fixture path for minimal_family.soft (1 GPL, 2 GSMs, 1 GSE)
fn fixture_minimal_family() -> String {
    format!(
        "{}/crates/geo-soft-rs/tests/fixtures/minimal_family.soft",
        env!("CARGO_MANIFEST_DIR")
            .split("/crates/transcriptomic-rs")
            .next()
            .unwrap()
    )
}

/// Fixture path for dual_channel.soft (1 GPL, 2 dual-channel GSMs)
fn fixture_dual_channel() -> String {
    format!(
        "{}/crates/geo-soft-rs/tests/fixtures/dual_channel.soft",
        env!("CARGO_MANIFEST_DIR")
            .split("/crates/transcriptomic-rs")
            .next()
            .unwrap()
    )
}

#[test]
fn test_matrix_assembly_minimal_family() {
    let path = fixture_minimal_family();
    let reader = SoftReader::open(&path).expect("Failed to open fixture");
    let matrix = MatrixBuilder::new()
        .from_soft(reader)
        .expect("Failed to build matrix");

    // Should have samples from the fixture
    assert!(!matrix.genes.is_empty(), "Should have genes");
    assert!(!matrix.samples.is_empty(), "Should have samples");

    // Check matrix dimensions
    assert_eq!(
        matrix.values.num_rows(),
        matrix.genes.len(),
        "Number of rows must match gene count"
    );
    assert_eq!(
        matrix.values.num_columns(),
        matrix.samples.len(),
        "Number of columns must match sample count"
    );
}

#[test]
fn test_matrix_assembly_dual_channel() {
    let path = fixture_dual_channel();
    let reader = SoftReader::open(&path).expect("Failed to open fixture");
    let matrix = MatrixBuilder::new()
        .from_soft(reader)
        .expect("Failed to build matrix");

    assert!(
        !matrix.genes.is_empty(),
        "Should have genes from dual-channel samples"
    );
    assert_eq!(
        matrix.values.num_rows(),
        matrix.genes.len(),
        "Dual-channel matrix should have rows matching gene count"
    );
}

#[test]
fn test_matrix_get_method() {
    let path = fixture_minimal_family();
    let reader = SoftReader::open(&path).expect("Failed to open fixture");
    let matrix = MatrixBuilder::new()
        .from_soft(reader)
        .expect("Failed to build matrix");

    // Test get method with valid indices
    if !matrix.genes.is_empty() && !matrix.samples.is_empty() {
        let gene = &matrix.genes[0];
        let sample = &matrix.samples[0];

        // get() should return Some or None depending on the data
        let _val = matrix.get(gene, sample);
        // Just verify it doesn't panic
    }

    // Test with invalid indices
    assert_eq!(
        matrix.get("NONEXISTENT_GENE", &matrix.samples[0]),
        None,
        "get() should return None for nonexistent gene"
    );
    assert_eq!(
        matrix.get(&matrix.genes[0], "NONEXISTENT_SAMPLE"),
        None,
        "get() should return None for nonexistent sample"
    );
}

#[test]
fn test_aggregation_methods() {
    let path = fixture_minimal_family();

    // Test with Mean aggregation (default)
    {
        let reader = SoftReader::open(&path).expect("Failed to open fixture");
        let matrix = MatrixBuilder::new()
            .from_soft(reader)
            .expect("Failed with Mean aggregation");
        assert!(!matrix.genes.is_empty());
    }

    // Test with Median aggregation
    {
        let reader = SoftReader::open(&path).expect("Failed to open fixture");
        let config = MatrixConfig {
            aggregation: AggregationMethod::Median,
            min_sample_presence: 1,
        };
        let matrix = MatrixBuilder::with_config(config)
            .from_soft(reader)
            .expect("Failed with Median aggregation");
        assert!(!matrix.genes.is_empty());
    }

    // Test with Max aggregation
    {
        let reader = SoftReader::open(&path).expect("Failed to open fixture");
        let config = MatrixConfig {
            aggregation: AggregationMethod::Max,
            min_sample_presence: 1,
        };
        let matrix = MatrixBuilder::with_config(config)
            .from_soft(reader)
            .expect("Failed with Max aggregation");
        assert!(!matrix.genes.is_empty());
    }

    // Test with Min aggregation
    {
        let reader = SoftReader::open(&path).expect("Failed to open fixture");
        let config = MatrixConfig {
            aggregation: AggregationMethod::Min,
            min_sample_presence: 1,
        };
        let matrix = MatrixBuilder::with_config(config)
            .from_soft(reader)
            .expect("Failed with Min aggregation");
        assert!(!matrix.genes.is_empty());
    }
}

#[test]
fn test_build_all_method() {
    let path = fixture_minimal_family();
    let reader = SoftReader::open(&path).expect("Failed to open fixture");

    let (matrix, metadata, annotation) = MatrixBuilder::new()
        .build_all(reader)
        .expect("Failed to build_all");

    assert!(!matrix.genes.is_empty());
    assert_eq!(matrix.values.num_columns(), matrix.samples.len());
    assert!(metadata.data.num_rows() > 0);
    // Platform annotation may be None if fixture has no platform
    let _ = annotation;
}
