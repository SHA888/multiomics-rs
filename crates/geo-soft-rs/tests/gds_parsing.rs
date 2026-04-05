//! Test GDS entity parsing

use geo_soft_rs::SoftReader;

#[test]
fn test_gds_parsing() -> geo_soft_rs::Result<()> {
    let soft_content = r#"^DATASET = GDS329
!dataset_title = Test Dataset
!dataset_description = This is a test dataset
!dataset_table_begin
ID_REF	IDENTIFIER	GSM1234	GSM1235
#ID_REF = Reference identifier
#IDENTIFIER = Gene identifier
probe1	gene1	1.5	2.1
probe2	gene2	0.8	1.3
!dataset_table_end
^SUBSET = GDS329_1
!subset_description = Control samples
!subset_sample_id = GSM1234
!subset_type = control
"#;

    // Create a temporary file
    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_gds.soft");
    std::fs::write(&temp_file, soft_content)?;

    // Parse the file
    let mut reader = SoftReader::open(&temp_file)?;

    // Try to get the first series (should be None since we only have GDS)
    let series = reader.next_series();
    assert!(series.is_none(), "Should not have any GSE records");

    // Clean up
    std::fs::remove_file(&temp_file)?;
    Ok(())
}
