//! Integration test using official GDS6063 dataset
//! This test verifies parsing of a real-world SOFT file with multiple subsets

use std::path::PathBuf;

fn resource_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../resources/geo");
    path.push(name);
    path
}

/// Test parsing of official GDS6063 dataset
/// This dataset has 7 subsets (2 infection groups + 5 individual donors)
#[test]
#[ignore = "Requires large official GDS file - run manually"]
fn test_official_gds6063_parsing() -> geo_soft_rs::Result<()> {
    let path = resource_path("GDS6063_full.soft");

    // Skip test if file doesn't exist (CI environments)
    if !path.exists() {
        eprintln!("Skipping test: GDS6063_full.soft not found at {:?}", path);
        return Ok(());
    }

    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    // Parse the dataset
    let datasets: Vec<_> = reader.datasets().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(datasets.len(), 1, "Should have exactly one GDS");

    let gds = &datasets[0];

    // Verify GDS metadata
    assert_eq!(gds.geo_accession, "GDS6063");
    assert_eq!(
        gds.title,
        "Influenza A virus effect on plasmacytoid dendritic cells"
    );
    assert_eq!(gds.platform, "GPL10558");
    assert_eq!(gds.sample_organism, "Homo sapiens");

    // Verify sample count (10 samples: 5 donors × 2 conditions)
    assert_eq!(gds.sample_count, 10);

    // Verify subsets - 7 total: 2 infection + 5 individual
    assert_eq!(gds.subsets.len(), 7);

    // Check infection subsets
    let infection_subsets: Vec<_> = gds
        .subsets
        .iter()
        .filter(|s| s.subset_type == "infection")
        .collect();
    assert_eq!(infection_subsets.len(), 2);

    // Verify influenza subset has 5 samples
    let influenza_subset = gds
        .subsets
        .iter()
        .find(|s| s.description == "influenza A")
        .expect("Should have influenza A subset");
    assert_eq!(influenza_subset.sample_ids.len(), 5);

    // Verify no virus control subset has 5 samples
    let control_subset = gds
        .subsets
        .iter()
        .find(|s| s.description == "no virus control")
        .expect("Should have no virus control subset");
    assert_eq!(control_subset.sample_ids.len(), 5);

    // Check individual (donor) subsets
    let individual_subsets: Vec<_> = gds
        .subsets
        .iter()
        .filter(|s| s.subset_type == "individual")
        .collect();
    assert_eq!(individual_subsets.len(), 5);

    // Each donor subset should have 2 samples (1 treated + 1 control)
    for subset in &individual_subsets {
        assert_eq!(subset.sample_ids.len(), 2);
    }

    // Verify data table exists
    assert!(gds.data_table.is_some(), "GDS should have data table");

    let table = gds.data_table.as_ref().unwrap();
    // Should have columns: ID_REF, IDENTIFIER, 10 sample columns, plus annotation
    // columns
    assert!(
        table.columns.len() > 12,
        "Should have ID_REF + IDENTIFIER + 10 samples + annotations"
    );

    // Verify feature count (number of rows/probes)
    assert!(gds.feature_count > 0);
    assert_eq!(table.rows.len(), gds.feature_count as usize);

    // Verify column names include sample columns
    let column_names: Vec<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
    for col in &["GSM1684096", "GSM1684095"] {
        assert!(column_names.contains(col), "Should have column for {}", col);
    }

    Ok(())
}

/// Test that GDS6063 can be converted to RecordBatch
#[test]
#[ignore = "Requires large official GDS file - run manually"]
fn test_gds6063_to_record_batch() -> geo_soft_rs::Result<()> {
    let path = resource_path("GDS6063_full.soft");

    if !path.exists() {
        eprintln!("Skipping test: GDS6063_full.soft not found");
        return Ok(());
    }

    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let datasets: Vec<_> = reader.datasets().collect::<Result<Vec<_>, _>>()?;

    if datasets.is_empty() {
        panic!("No datasets found");
    }

    let batch = datasets[0].to_record_batch()?;

    // Verify schema
    let schema = batch.schema();
    assert!(schema.fields().len() > 10, "Should have many columns");

    // Verify row count matches feature_count
    assert_eq!(batch.num_rows(), datasets[0].feature_count as usize);

    // Check required metadata keys
    let metadata = schema.metadata();
    assert!(metadata.contains_key("geo_accession"));
    assert!(metadata.contains_key("geo_channel_count"));
    assert!(metadata.contains_key("geo_platform_id"));

    assert_eq!(metadata["geo_accession"], "GDS6063");
    assert_eq!(metadata["geo_channel_count"], "1");
    assert_eq!(metadata["geo_platform_id"], "GPL10558");

    Ok(())
}

/// Test read_all() with official GDS file
#[test]
#[ignore = "Requires large official GDS file - run manually"]
fn test_gds6063_read_all() -> geo_soft_rs::Result<()> {
    let path = resource_path("GDS6063_full.soft");

    if !path.exists() {
        eprintln!("Skipping test: GDS6063_full.soft not found");
        return Ok(());
    }

    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let file = reader.read_all()?;

    // Should have datasets
    assert_eq!(file.datasets.len(), 1);

    // In a GDS file, we primarily expect datasets (platforms/samples are embedded)
    // Just verify we successfully parsed without error

    Ok(())
}

/// Quick smoke test - just verify file opens and has expected structure
#[test]
fn test_gds6063_smoke() {
    let path = resource_path("GDS6063_full.soft");

    if !path.exists() {
        eprintln!("Skipping test: GDS6063_full.soft not found");
        return;
    }

    // Should open without error
    let result = geo_soft_rs::SoftReader::open(&path);
    assert!(
        result.is_ok(),
        "Should open GDS6063 without error: {:?}",
        result.err()
    );
}
