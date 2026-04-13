//! Comprehensive tests for SoftReader API (G1.4)

use std::path::PathBuf;

use arrow::array::Array;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    path
}

// G1.4.2: Entity header parsing tests
#[test]
fn test_entity_header_parsing() -> geo_soft_rs::Result<()> {
    // Each entity type test uses a separate reader since they share the underlying
    // stream

    // Test series parsing
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let series = reader.series().next().expect("Should have series")?;
    assert_eq!(series.local_id, "GSETEST1");
    assert_eq!(series.geo_accession, Some("GSETEST1".to_string()));

    // Test samples - separate reader
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].local_id, "GSMTEST1");
    assert_eq!(samples[0].geo_accession, Some("GSMTEST1".to_string()));
    assert_eq!(samples[1].local_id, "GSMTEST2");

    // Test platform - separate reader
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let platforms: Vec<_> = reader.platforms().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(platforms.len(), 1);
    assert_eq!(platforms[0].local_id, "GPLTEST1");

    Ok(())
}

// G1.4.2: local_id vs geo_accession differ
#[test]
fn test_local_id_vs_geo_accession() -> geo_soft_rs::Result<()> {
    // Create a fixture with different local_id and geo_accession
    let soft_content = r#"^SAMPLE = my_local_name
!Sample_title = Test Sample
!Sample_geo_accession = GSM99999
!Sample_platform_id = GPLTEST
!Sample_channel_count = 1
!Sample_table_begin
ID_REF	VALUE
probe1	1.0
!Sample_table_end
"#;

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_local_id.soft");
    std::fs::write(&temp_file, soft_content)?;

    let mut reader = geo_soft_rs::SoftReader::open(&temp_file)?;
    let sample = reader.samples().next().expect("Should have sample")?;

    // Verify they are NOT conflated
    assert_eq!(sample.local_id, "my_local_name");
    assert_eq!(sample.geo_accession, Some("GSM99999".to_string()));

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

// G1.4.2: Metadata parsing - multi-value fields accumulated correctly
#[test]
fn test_multi_value_metadata() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let series = reader.series().next().expect("Should have series")?;

    // Series has two sample_id entries
    assert_eq!(series.sample_ids.len(), 2);
    assert!(series.sample_ids.contains(&"GSMTEST1".to_string()));
    assert!(series.sample_ids.contains(&"GSMTEST2".to_string()));

    Ok(())
}

// G1.4.2: Hash lines populate ColumnDescriptor.description
#[test]
fn test_hash_lines_column_descriptor() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let platform = reader.platforms().next().expect("Should have platform")?;

    // Platform should have annotation table with column descriptors
    let table = platform.annotation_table.expect("Should have table");

    // Check that hash lines were parsed into column descriptors
    // The columns vec contains ColumnDescriptor with name and description
    let column_names: Vec<_> = table.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.contains(&"ID_REF"));
    assert!(column_names.contains(&"IDENTIFIER"));
    assert!(column_names.contains(&"Gene_Symbol"));

    // Check that descriptions were populated from hash lines
    let id_ref_col = table.columns.iter().find(|c| c.name == "ID_REF").unwrap();
    assert!(!id_ref_col.description.is_empty());

    Ok(())
}

// G1.4.2: Table parsing - column descriptors + row values
#[test]
fn test_table_parsing() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let platform = reader.platforms().next().expect("Should have platform")?;
    let table = platform.annotation_table.expect("Should have table");

    // Check columns
    assert_eq!(table.columns.len(), 3);

    // Check rows
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0][0], "probe1");
    assert_eq!(table.rows[0][1], "gene1");

    Ok(())
}

// G1.4.2: gzip - same output as uncompressed
#[test]
fn test_gzip_parsing() -> geo_soft_rs::Result<()> {
    // Wait for gzip to finish
    std::thread::sleep(std::time::Duration::from_millis(100));

    let plain_path = fixture_path("minimal_family.soft");
    let gz_path = fixture_path("minimal_family.soft.gz");

    // Parse plain file
    let mut plain_reader = geo_soft_rs::SoftReader::open(&plain_path)?;
    let plain_series: Vec<_> = plain_reader.series().collect::<Result<Vec<_>, _>>()?;

    // Parse gzipped file
    let mut gz_reader = geo_soft_rs::SoftReader::open_gz(&gz_path)?;
    let gz_series: Vec<_> = gz_reader.series().collect::<Result<Vec<_>, _>>()?;

    // Should have same results
    assert_eq!(plain_series.len(), gz_series.len());
    assert_eq!(plain_series[0].local_id, gz_series[0].local_id);

    // Check sample counts match
    let plain_samples: Vec<_> = plain_reader.samples().collect::<Result<Vec<_>, _>>()?;
    let gz_samples: Vec<_> = gz_reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(plain_samples.len(), gz_samples.len());

    Ok(())
}

// G1.4.3: to_record_batch() schema matches declared schema
#[test]
fn test_record_batch_schema() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert!(!samples.is_empty());

    let batch = samples[0].to_record_batch()?;
    let schema = batch.schema();

    // Check expected fields exist (lowercase as per implementation)
    let fields: Vec<_> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert!(fields.contains(&"id_ref"));
    assert!(fields.contains(&"value"));

    Ok(())
}

// G1.4.3: Schema metadata keys present
#[test]
fn test_schema_metadata() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    let batch = samples[0].to_record_batch()?;
    let schema = batch.schema();
    let metadata = schema.metadata();

    // Check required metadata keys
    assert!(metadata.contains_key("geo_accession"));
    assert!(metadata.contains_key("geo_channel_count"));
    assert!(metadata.contains_key("geo_platform_id"));

    assert_eq!(metadata["geo_accession"], "GSMTEST1");
    assert_eq!(metadata["geo_channel_count"], "1");
    assert_eq!(metadata["geo_platform_id"], "GPLTEST1");

    Ok(())
}

// G1.4.4: Integration test - parse minimal_family.soft end-to-end
#[test]
fn test_integration_minimal_family() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let file = reader.read_all()?;

    // Assert series accession
    assert_eq!(file.series.len(), 1);
    assert_eq!(file.series[0].local_id, "GSETEST1");

    // Assert sample count
    assert_eq!(file.samples.len(), 2);

    // Assert platform annotation row count
    assert_eq!(file.platforms.len(), 1);
    let platform = &file.platforms[0];
    let table = platform
        .annotation_table
        .as_ref()
        .expect("Should have table");
    assert_eq!(table.rows.len(), 3);

    Ok(())
}

// G1.4.5: Empty tables - no panic
#[test]
fn test_empty_table_handling() -> geo_soft_rs::Result<()> {
    let soft_content = r#"^SAMPLE = GSMEMPTY
!Sample_title = Empty Table Sample
!Sample_geo_accession = GSMEMPTY
!Sample_platform_id = GPLTEST
!Sample_channel_count = 1
!Sample_table_begin
ID_REF	VALUE
!Sample_table_end
"#;

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_empty_table.soft");
    std::fs::write(&temp_file, soft_content)?;

    let mut reader = geo_soft_rs::SoftReader::open(&temp_file)?;
    // Should not panic on empty table
    let _sample = reader.samples().next().expect("Should have sample")?;

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

// G1.4.5: Missing fields - no panic
#[test]
fn test_missing_fields_handling() -> geo_soft_rs::Result<()> {
    let soft_content = r#"^SAMPLE = GSMMINIMAL
!Sample_title = Minimal Sample
!Sample_platform_id = GPLTEST
!Sample_channel_count = 1
!Sample_table_begin
ID_REF	VALUE
probe1	1.0
!Sample_table_end
"#;

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_missing_fields.soft");
    std::fs::write(&temp_file, soft_content)?;

    let mut reader = geo_soft_rs::SoftReader::open(&temp_file)?;
    let sample = reader.samples().next().expect("Should have sample")?;

    // geo_accession is missing but should be None, not error
    assert_eq!(sample.geo_accession, None);
    assert_eq!(sample.local_id, "GSMMINIMAL");

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

// G1.4.6: Dual-channel fixture - channel_count = 2
#[test]
fn test_dual_channel_parsing() -> geo_soft_rs::Result<()> {
    let path = fixture_path("dual_channel.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(samples.len(), 2);

    // Check channel_count = 2
    assert_eq!(samples[0].channel_count, 2);

    // Check schema has ch1_value and ch2_value columns
    let batch = samples[0].to_record_batch()?;
    let schema = batch.schema();
    // Should have value, ch1_value, ch2_value columns (lowercase)
    let fields: Vec<_> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert!(fields.contains(&"value"));
    assert!(fields.contains(&"ch1_value"));
    assert!(fields.contains(&"ch2_value"));

    Ok(())
}

// G1.4.7: GDS fixture - GdsRecord parsed, GdsSubset list populated
#[test]
fn test_gds_with_subsets() -> geo_soft_rs::Result<()> {
    let path = fixture_path("gds_with_subsets.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let datasets: Vec<_> = reader.datasets().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(datasets.len(), 1);

    let gds = &datasets[0];
    assert_eq!(gds.geo_accession, "GDSTEST1");
    assert_eq!(gds.feature_count, 3);
    assert_eq!(gds.sample_count, 4);

    // Check subsets populated
    assert_eq!(gds.subsets.len(), 2);

    // Check first subset
    assert_eq!(gds.subsets[0].local_id, "GDSTEST1_control");
    assert_eq!(gds.subsets[0].subset_type, "disease_state");

    Ok(())
}

// G1.4.8: Download-attrs fixture - route to metadata HashMap
#[test]
fn test_download_attrs_routing() -> geo_soft_rs::Result<()> {
    // Test series - separate reader
    let path = fixture_path("download_attrs.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let series = reader.series().next().expect("Should have series")?;

    // Download attrs should be in metadata HashMap, not in named struct fields
    assert!(series.metadata.contains_key("status"));
    assert!(series.metadata.contains_key("submission_date"));
    assert!(series.metadata.contains_key("contact_name"));

    // These should NOT be in the named fields
    assert!(series.title.is_empty() || !series.title.contains("status"));

    // Same for samples - separate reader
    let path = fixture_path("download_attrs.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert!(!samples.is_empty());
    assert!(samples[0].metadata.contains_key("status"));
    assert!(samples[0].metadata.contains_key("contact_name"));

    Ok(())
}

// G1.4.9: Null sentinel coverage
#[test]
fn test_null_sentinels() -> geo_soft_rs::Result<()> {
    let soft_content = include_str!("fixtures/null_sentinels.soft");

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_nulls.soft");
    std::fs::write(&temp_file, soft_content)?;

    let mut reader = geo_soft_rs::SoftReader::open(&temp_file)?;
    let sample = reader.samples().next().expect("Should have sample")?;
    let batch = sample.to_record_batch()?;

    let value_col = batch
        .column(1)
        .as_any()
        .downcast_ref::<arrow::array::Float64Array>()
        .expect("Should be Float64");

    // All should be null except the last one
    assert!(value_col.is_null(0)); // empty
    assert!(value_col.is_null(1)); // NA
    assert!(value_col.is_null(2)); // null
    assert!(value_col.is_null(3)); // NaN
    assert!(value_col.is_null(4)); // none
    assert!(!value_col.is_null(5)); // 1.5
    assert_eq!(value_col.value(5), 1.5);

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

// G1.4.10: Malformed float - should return Err
#[test]
fn test_malformed_float_error() -> geo_soft_rs::Result<()> {
    let soft_content = r#"^SAMPLE = GSMERR
!Sample_title = Error Test
!Sample_geo_accession = GSMERR
!Sample_platform_id = GPLTEST
!Sample_channel_count = 1
!Sample_table_begin
ID_REF	VALUE
probe1	abc
!Sample_table_end
"#;

    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_error.soft");
    std::fs::write(&temp_file, soft_content)?;

    let mut reader = geo_soft_rs::SoftReader::open(&temp_file)?;
    let sample = reader.samples().next().expect("Should have sample")?;

    // Should return Err, not Ok with None
    let result = sample.to_record_batch();
    assert!(result.is_err());

    std::fs::remove_file(&temp_file)?;
    Ok(())
}

// G1.4.11: Line endings - CRLF vs LF produces identical results
#[test]
fn test_line_endings() -> geo_soft_rs::Result<()> {
    let lf_content = "^SAMPLE = GSMLF\n!Sample_title = LF Test\n!Sample_platform_id = GPLTEST\n!Sample_channel_count = 1\n!Sample_table_begin\nID_REF\tVALUE\nprobe1\t1.0\n!Sample_table_end\n";
    let crlf_content = "^SAMPLE = GSMCRLF\r\n!Sample_title = CRLF Test\r\n!Sample_platform_id = GPLTEST\r\n!Sample_channel_count = 1\r\n!Sample_table_begin\r\nID_REF\tVALUE\r\nprobe1\t1.0\r\n!Sample_table_end\r\n";

    let mut lf_file = std::env::temp_dir();
    lf_file.push("test_lf.soft");
    std::fs::write(&lf_file, lf_content)?;

    let mut crlf_file = std::env::temp_dir();
    crlf_file.push("test_crlf.soft");
    std::fs::write(&crlf_file, crlf_content)?;

    let mut lf_reader = geo_soft_rs::SoftReader::open(&lf_file)?;
    let mut crlf_reader = geo_soft_rs::SoftReader::open(&crlf_file)?;

    let lf_sample = lf_reader.samples().next().expect("Should have sample")?;
    let crlf_sample = crlf_reader.samples().next().expect("Should have sample")?;

    // Both should have same data
    assert_eq!(lf_sample.title, "LF Test");
    assert_eq!(crlf_sample.title, "CRLF Test");
    assert_eq!(lf_sample.channel_count, crlf_sample.channel_count);

    let lf_batch = lf_sample.to_record_batch()?;
    let crlf_batch = crlf_sample.to_record_batch()?;

    // Same row count
    assert_eq!(lf_batch.num_rows(), crlf_batch.num_rows());

    std::fs::remove_file(&lf_file)?;
    std::fs::remove_file(&crlf_file)?;
    Ok(())
}
