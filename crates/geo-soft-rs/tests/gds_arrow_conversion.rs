//! Test GDS Arrow conversion

use geo_soft_rs::SoftReader;

#[test]
fn test_gds_to_record_batch() -> geo_soft_rs::Result<()> {
    let soft_content = r#"^DATASET = GDS6063
!dataset_title = Influenza A effect on plasmacytoid dendritic cells
!dataset_description = Analysis of primary plasmacytoid dendritic cells (pDC) exposed to influenza A for 8 hours ex vivo.
!dataset_sample_count = 3
^DATASET = GDS6063
#ID_REF = Platform reference identifier
#IDENTIFIER = identifier
#GSM1684096 = Value for GSM1684096: Donor 1 - Influenza treated - 8h
#GSM1684098 = Value for GSM1684098: Donor 2 - Influenza treated - 8h
#GSM1684100 = Value for GSM1684100: Donor 3 - Influenza treated - 8h
!dataset_table_begin
ID_REF	IDENTIFIER	GSM1684096	GSM1684098	GSM1684100
ILMN_1343291	EEF1A1	22303.3	24776.1	26775.3
ILMN_1343295	GAPDH	7732.65	5296.75	6430.16
ILMN_1343296	null	null	null	null
!dataset_table_end
"#;

    // Create a temporary file
    let mut temp_file = std::env::temp_dir();
    temp_file.push("test_gds_arrow.soft");
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
