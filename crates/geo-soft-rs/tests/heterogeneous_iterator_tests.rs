//! Tests for heterogeneous records() iterator (G1.3.7)

use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    path
}

#[test]
fn test_heterogeneous_iterator_preserves_order() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    // Collect all records in file order
    let records: Vec<_> = reader.records().collect::<Result<Vec<_>, _>>()?;

    // Should have: Platform, Sample1, Sample2, Series
    assert_eq!(records.len(), 4);

    // Check order is preserved
    assert!(matches!(records[0], geo_soft_rs::SoftRecord::Platform(_)));
    assert!(matches!(records[1], geo_soft_rs::SoftRecord::Sample(_)));
    assert!(matches!(records[2], geo_soft_rs::SoftRecord::Sample(_)));
    assert!(matches!(records[3], geo_soft_rs::SoftRecord::Series(_)));

    // Verify specific accessions
    if let geo_soft_rs::SoftRecord::Platform(p) = &records[0] {
        assert_eq!(p.local_id, "GPLTEST1");
    }
    if let geo_soft_rs::SoftRecord::Sample(s) = &records[1] {
        assert_eq!(s.local_id, "GSMTEST1");
    }
    if let geo_soft_rs::SoftRecord::Series(gse) = &records[3] {
        assert_eq!(gse.local_id, "GSETEST1");
    }

    Ok(())
}

#[test]
fn test_multi_section_heterogeneous() -> geo_soft_rs::Result<()> {
    let path = fixture_path("multi_section.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let records: Vec<_> = reader.records().collect::<Result<Vec<_>, _>>()?;

    // Should have: Platform1, Platform2, Sample1, Sample2, Series1, Series2
    assert_eq!(records.len(), 6);

    // Check order
    assert!(matches!(records[0], geo_soft_rs::SoftRecord::Platform(_)));
    assert!(matches!(records[1], geo_soft_rs::SoftRecord::Platform(_)));
    assert!(matches!(records[2], geo_soft_rs::SoftRecord::Sample(_)));
    assert!(matches!(records[3], geo_soft_rs::SoftRecord::Sample(_)));
    assert!(matches!(records[4], geo_soft_rs::SoftRecord::Series(_)));
    assert!(matches!(records[5], geo_soft_rs::SoftRecord::Series(_)));

    Ok(())
}

#[test]
fn test_read_all_collects_all_types() -> geo_soft_rs::Result<()> {
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    let file = reader.read_all()?;

    // All types collected
    assert_eq!(file.platforms.len(), 1);
    assert_eq!(file.samples.len(), 2);
    assert_eq!(file.series.len(), 1);
    assert!(file.datasets.is_empty()); // No GDS in this fixture

    Ok(())
}

#[test]
fn test_entity_specific_iterators_filter_correctly() -> geo_soft_rs::Result<()> {
    // Note: Entity-specific iterators share the underlying reader.
    // Each test uses a separate reader since once an iterator is exhausted,
    // the reader is at EOF and other iterators would return nothing.

    // Test platforms() - uses separate reader
    let path = fixture_path("multi_section.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let platforms: Vec<_> = reader.platforms().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(platforms.len(), 2);
    assert_eq!(platforms[0].local_id, "GPLMULTI1");
    assert_eq!(platforms[1].local_id, "GPLMULTI2");

    // Test samples() - uses separate reader
    let path = fixture_path("multi_section.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let samples: Vec<_> = reader.samples().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(samples.len(), 2);
    assert_eq!(samples[0].local_id, "GSMMULTI1");
    assert_eq!(samples[1].local_id, "GSMMULTI2");

    // Test series() - uses separate reader
    let path = fixture_path("multi_section.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;
    let series: Vec<_> = reader.series().collect::<Result<Vec<_>, _>>()?;
    assert_eq!(series.len(), 2);
    assert_eq!(series[0].local_id, "GSEMULTI1");
    assert_eq!(series[1].local_id, "GSEMULTI2");

    Ok(())
}

#[test]
fn test_family_file_entity_order_unknown() -> geo_soft_rs::Result<()> {
    // In family files, entity order is unknown - records() handles this
    let path = fixture_path("minimal_family.soft");
    let mut reader = geo_soft_rs::SoftReader::open(&path)?;

    // Use records() to handle any order
    let records: Vec<_> = reader.records().collect::<Result<Vec<_>, _>>()?;

    // All entities present regardless of order
    let platform_count = records
        .iter()
        .filter(|r| matches!(r, geo_soft_rs::SoftRecord::Platform(_)))
        .count();
    let sample_count = records
        .iter()
        .filter(|r| matches!(r, geo_soft_rs::SoftRecord::Sample(_)))
        .count();
    let series_count = records
        .iter()
        .filter(|r| matches!(r, geo_soft_rs::SoftRecord::Series(_)))
        .count();

    assert_eq!(platform_count, 1);
    assert_eq!(sample_count, 2);
    assert_eq!(series_count, 1);

    Ok(())
}
