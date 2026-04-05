//! SOFT format parser

use std::{collections::HashMap, io::BufRead};

use arrow::record_batch::RecordBatch;

use crate::{Error, Result};

/// Parser state for SOFT format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Idle,
    InSeries,
    InPlatform,
    InSample,
    InDataset,
    InSubset,
    InPlatformTable,
    InSampleTable,
    InDatasetTable,
}

/// Reader for SOFT format files
pub struct SoftReader<R: BufRead> {
    reader: R,
    line_number: usize,
    state: ParseState,
    current_series: Option<GseRecord>,
    current_platform: Option<GplRecord>,
    current_sample: Option<GsmRecord>,
    current_dataset: Option<GdsRecord>,
    current_subset: Option<GdsSubset>,
    current_table: Option<DataTable>,
    eof_reached: bool,
}

impl<R: BufRead> SoftReader<R> {
    /// Create a new SOFT reader
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            line_number: 0,
            state: ParseState::Idle,
            current_series: None,
            current_platform: None,
            current_sample: None,
            current_dataset: None,
            current_subset: None,
            current_table: None,
            eof_reached: false,
        }
    }

    /// Iterate over GSE records
    #[must_use]
    pub fn series(&mut self) -> impl Iterator<Item = Result<GseRecord>> + '_ {
        std::iter::from_fn(move || self.next_series())
    }

    /// Get the next GSE record
    pub fn next_series(&mut self) -> Option<Result<GseRecord>> {
        // If EOF was reached and no series, return None
        if self.eof_reached && self.current_series.is_none() {
            return None;
        }

        loop {
            match self.parse_next_line() {
                Ok(Some(record)) => return Some(Ok(record)),
                Ok(None) => {
                    // Check if we have a pending series
                    if self.current_series.is_some() {
                        return self.current_series.take().map(Ok);
                    } else if self.eof_reached {
                        // No series and EOF reached - we're done
                        return None;
                    }
                    // No series yet, continue parsing
                    continue;
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }

    /// Parse the next line from the input
    fn parse_next_line(&mut self) -> Result<Option<GseRecord>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line)?;

        if bytes_read == 0 {
            // End of file - set flag and return any pending series
            self.eof_reached = true;
            return Ok(self.current_series.take());
        }

        self.line_number += 1;
        let line = line.trim_end();

        if line.is_empty() {
            return Ok(None);
        }

        // Handle different line types based on state
        match self.state {
            ParseState::Idle => self.handle_idle_state(line),
            ParseState::InSeries => self.handle_series_state(line),
            ParseState::InPlatform => self.handle_platform_state(line),
            ParseState::InSample => self.handle_sample_state(line),
            ParseState::InDataset => self.handle_dataset_state(line),
            ParseState::InSubset => self.handle_subset_state(line),
            ParseState::InPlatformTable => self.handle_platform_table_state(line),
            ParseState::InSampleTable => self.handle_sample_table_state(line),
            ParseState::InDatasetTable => self.handle_dataset_table_state(line),
        }
    }

    /// Handle lines when in idle state
    fn handle_idle_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if let Some(accession) = line.strip_prefix("^SERIES = ") {
            Ok(self.start_series(accession.trim()))
        } else if let Some(accession) = line.strip_prefix("^PLATFORM = ") {
            self.start_platform(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^SAMPLE = ") {
            self.start_sample(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^DATASET = ") {
            self.start_dataset(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^SUBSET = ") {
            self.start_subset(accession.trim())
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in series state
    fn handle_series_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section - return current series
            return Ok(self.current_series.take());
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_series_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in platform state
    fn handle_platform_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section
            if let Some(_) = line.strip_prefix("^PLATFORM = ") {
                // New platform - finish current one
                self.current_platform = None;
                Ok(None)
            } else {
                // Other entity - return None for now (we're only parsing series)
                Ok(None)
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_platform_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in sample state
    fn handle_sample_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section
            if let Some(_) = line.strip_prefix("^SAMPLE = ") {
                // New sample - finish current one
                self.current_sample = None;
                Ok(None)
            } else {
                // Other entity - return None for now
                Ok(None)
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_sample_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in dataset state
    fn handle_dataset_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section - finalize current dataset
            if let Some(_) = line.strip_prefix("^DATASET = ") {
                // New dataset - current one is lost (no storage for multiple datasets)
                self.current_dataset = None;
                Ok(None)
            } else {
                // Other entity - finalize current dataset
                self.current_dataset = None;
                Ok(None)
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_dataset_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in subset state
    fn handle_subset_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section - add completed subset to dataset
            if let Some(subset) = self.current_subset.take() {
                if let Some(dataset) = &mut self.current_dataset {
                    dataset.subsets.push(subset);
                }
            }

            if let Some(_) = line.strip_prefix("^SUBSET = ") {
                // New subset - start fresh
                self.start_subset(line.strip_prefix("^SUBSET = ").unwrap().trim())
            } else {
                // Other entity - return None for now
                Ok(None)
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_subset_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in platform table state
    fn handle_platform_table_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.to_lowercase() == "!platform_table_end" {
            if let Some(table) = self.current_table.take() {
                if let Some(platform) = &mut self.current_platform {
                    platform.annotation_table = Some(table);
                }
            }
            self.state = ParseState::InPlatform;
        } else if let Some(table) = &mut self.current_table {
            if table.columns.is_empty() {
                // First line is header
                let headers: Vec<String> = line.split('\t').map(String::from).collect();
                table.columns = headers
                    .into_iter()
                    .map(|h| ColumnDescriptor {
                        name: h.clone(),
                        description: String::new(),
                    })
                    .collect();
            } else if line.starts_with('#') {
                // Hash line provides column description
                if let Some((col_name, desc)) =
                    line.strip_prefix('#').and_then(|l| l.split_once("="))
                {
                    let col_name = col_name.trim();
                    let desc = desc.trim();
                    if let Some(col) = table.columns.iter_mut().find(|c| c.name == col_name) {
                        col.description = desc.to_string();
                    }
                }
            } else {
                // Data row
                let fields: Vec<String> = line.split('\t').map(String::from).collect();
                if fields.len() != table.columns.len() {
                    return Err(Error::Parse {
                        line: self.line_number,
                        message: format!(
                            "Column count mismatch: expected {}, got {}",
                            table.columns.len(),
                            fields.len()
                        ),
                    });
                }
                table.rows.push(fields);
            }
        }
        Ok(None)
    }

    /// Handle lines when in sample table state
    fn handle_sample_table_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.to_lowercase() == "!sample_table_end" {
            if let Some(table) = self.current_table.take() {
                if let Some(sample) = &mut self.current_sample {
                    sample.data_table = Some(table);
                }
            }
            self.state = ParseState::InSample;
        } else if let Some(table) = &mut self.current_table {
            if table.columns.is_empty() {
                // First line is header
                let headers: Vec<String> = line.split('\t').map(String::from).collect();
                table.columns = headers
                    .into_iter()
                    .map(|h| ColumnDescriptor {
                        name: h.clone(),
                        description: String::new(),
                    })
                    .collect();
            } else if line.starts_with('#') {
                // Hash line provides column description
                if let Some((col_name, desc)) =
                    line.strip_prefix('#').and_then(|l| l.split_once("="))
                {
                    let col_name = col_name.trim();
                    let desc = desc.trim();
                    if let Some(col) = table.columns.iter_mut().find(|c| c.name == col_name) {
                        col.description = desc.to_string();
                    }
                }
            } else {
                // Data row
                let fields: Vec<String> = line.split('\t').map(String::from).collect();
                if fields.len() != table.columns.len() {
                    return Err(Error::Parse {
                        line: self.line_number,
                        message: format!(
                            "Column count mismatch: expected {}, got {}",
                            table.columns.len(),
                            fields.len()
                        ),
                    });
                }
                table.rows.push(fields);
            }
        }
        Ok(None)
    }

    /// Handle lines when in dataset table state
    fn handle_dataset_table_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.to_lowercase() == "!dataset_table_end" {
            if let Some(table) = self.current_table.take() {
                if let Some(dataset) = &mut self.current_dataset {
                    // Copy column descriptions to dataset
                    for col in &table.columns {
                        if !col.description.is_empty() {
                            dataset
                                .column_descs
                                .insert(col.name.clone(), col.description.clone());
                        }
                    }
                    dataset.data_table = Some(table);
                }
            }
            self.state = ParseState::InDataset;
        } else if let Some(table) = &mut self.current_table {
            if table.columns.is_empty() {
                // First line is header
                let headers: Vec<String> = line.split('\t').map(String::from).collect();
                table.columns = headers
                    .into_iter()
                    .map(|h| ColumnDescriptor {
                        name: h.clone(),
                        description: String::new(),
                    })
                    .collect();
            } else if line.starts_with('#') {
                // Hash line provides column description
                if let Some((col_name, desc)) =
                    line.strip_prefix('#').and_then(|l| l.split_once("="))
                {
                    let col_name = col_name.trim();
                    let desc = desc.trim();
                    if let Some(col) = table.columns.iter_mut().find(|c| c.name == col_name) {
                        col.description = desc.to_string();
                    }
                }
            } else {
                // Data row
                let fields: Vec<String> = line.split('\t').map(String::from).collect();
                if fields.len() != table.columns.len() {
                    return Err(Error::Parse {
                        line: self.line_number,
                        message: format!(
                            "Column count mismatch: expected {}, got {}",
                            table.columns.len(),
                            fields.len()
                        ),
                    });
                }
                table.rows.push(fields);
            }
        }
        Ok(None)
    }

    /// Start parsing a new series
    fn start_series(&mut self, accession: &str) -> Option<GseRecord> {
        self.state = ParseState::InSeries;
        self.current_series = Some(GseRecord {
            accession: accession.to_string(),
            title: String::new(),
            summary: String::new(),
            overall_design: String::new(),
            submission_date: chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()),
            sample_ids: Vec::new(),
            platform_ids: Vec::new(),
            metadata: HashMap::new(),
        });
        None
    }

    /// Start parsing a new platform
    fn start_platform(&mut self, accession: &str) -> Result<Option<GseRecord>> {
        self.state = ParseState::InPlatform;
        self.current_platform = Some(GplRecord {
            accession: accession.to_string(),
            title: String::new(),
            technology: String::new(),
            annotation_table: None,
        });
        Ok(None)
    }

    /// Start parsing a new sample
    fn start_sample(&mut self, accession: &str) -> Result<Option<GseRecord>> {
        self.state = ParseState::InSample;
        self.current_sample = Some(GsmRecord {
            accession: accession.to_string(),
            title: String::new(),
            characteristics: HashMap::new(),
            platform_id: String::new(),
            data_table: None,
        });
        Ok(None)
    }

    /// Start parsing a new dataset
    fn start_dataset(&mut self, accession: &str) -> Result<Option<GseRecord>> {
        self.state = ParseState::InDataset;
        self.current_dataset = Some(GdsRecord {
            geo_accession: accession.to_string(),
            title: String::new(),
            description: String::new(),
            platform: String::new(),
            sample_organism: String::new(),
            sample_type: String::new(),
            feature_count: 0,
            sample_count: 0,
            subsets: Vec::new(),
            metadata: HashMap::new(),
            column_descs: HashMap::new(),
            data_table: None,
        });
        Ok(None)
    }

    /// Start parsing a new subset
    fn start_subset(&mut self, accession: &str) -> Result<Option<GseRecord>> {
        self.state = ParseState::InSubset;
        self.current_subset = Some(GdsSubset {
            local_id: accession.to_string(),
            description: String::new(),
            sample_ids: Vec::new(),
            subset_type: String::new(),
        });
        Ok(None)
    }

    /// Parse series metadata
    fn parse_series_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(series) = &mut self.current_series {
                match key {
                    "Series_title" => series.title = value.to_string(),
                    "Series_summary" => series.summary = value.to_string(),
                    "Series_overall_design" => series.overall_design = value.to_string(),
                    "Series_submission_date" => {
                        series.submission_date =
                            chrono::NaiveDate::parse_from_str(value, "%b %d %Y")
                                .or_else(|_| chrono::NaiveDate::parse_from_str(value, "%B %d %Y"))
                                .or_else(|_| chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d"))
                                .unwrap_or_else(|_| {
                                    chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
                                });
                    }
                    "Series_sample_id" => series.sample_ids.push(value.to_string()),
                    "Series_platform_id" => series.platform_ids.push(value.to_string()),
                    _ => {
                        series
                            .metadata
                            .entry(key.to_string())
                            .or_insert_with(Vec::new)
                            .push(value.to_string());
                    }
                }
            }
        }
        Ok(None)
    }

    /// Parse platform metadata
    fn parse_platform_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(platform) = &mut self.current_platform {
                match key {
                    "Platform_title" => platform.title = value.to_string(),
                    "Platform_technology" => platform.technology = value.to_string(),
                    "!Platform_table_begin" => {
                        self.state = ParseState::InPlatformTable;
                        self.current_table = Some(DataTable {
                            columns: Vec::new(),
                            rows: Vec::new(),
                        });
                    }
                    _ => {} // Handle other platform metadata as needed
                }
            }
        }
        Ok(None)
    }

    /// Parse sample metadata
    fn parse_sample_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(sample) = &mut self.current_sample {
                match key {
                    "Sample_title" => sample.title = value.to_string(),
                    "Sample_platform_id" => sample.platform_id = value.to_string(),
                    "!sample_table_begin" => {
                        self.state = ParseState::InSampleTable;
                        self.current_table = Some(DataTable {
                            columns: Vec::new(),
                            rows: Vec::new(),
                        });
                    }
                    key if key.starts_with("Sample_characteristics_") => {
                        // Extract characteristics type from the key
                        if let Some(char_type) = key.strip_prefix("Sample_characteristics_") {
                            sample
                                .characteristics
                                .insert(char_type.to_string(), value.to_string());
                        }
                    }
                    _ => {} // Handle other sample metadata as needed
                }
            }
        }
        Ok(None)
    }

    /// Parse dataset metadata
    fn parse_dataset_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(dataset) = &mut self.current_dataset {
                match key {
                    "dataset_title" => dataset.title = value.to_string(),
                    "dataset_description" => dataset.description = value.to_string(),
                    "dataset_platform" => dataset.platform = value.to_string(),
                    "dataset_sample_organism" => dataset.sample_organism = value.to_string(),
                    "dataset_sample_type" => dataset.sample_type = value.to_string(),
                    "dataset_feature_count" => {
                        if let Ok(count) = value.parse::<u32>() {
                            dataset.feature_count = count;
                        }
                    }
                    "dataset_sample_count" => {
                        if let Ok(count) = value.parse::<u32>() {
                            dataset.sample_count = count;
                        }
                    }
                    "!dataset_table_begin" => {
                        self.state = ParseState::InDatasetTable;
                        self.current_table = Some(DataTable {
                            columns: Vec::new(),
                            rows: Vec::new(),
                        });
                    }
                    _ => {
                        dataset
                            .metadata
                            .entry(key.to_string())
                            .or_insert_with(Vec::new)
                            .push(value.to_string());
                    }
                }
            }
        }
        Ok(None)
    }

    /// Parse subset metadata
    fn parse_subset_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(subset) = &mut self.current_subset {
                match key {
                    "subset_description" => subset.description = value.to_string(),
                    "subset_type" => subset.subset_type = value.to_string(),
                    "subset_sample_id" => {
                        // Handle comma-separated sample IDs
                        for sample_id in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                            subset.sample_ids.push(sample_id.to_string());
                        }
                    }
                    _ => {} // Handle other subset metadata as needed
                }
            }
        }
        Ok(None)
    }
}

impl SoftReader<std::io::BufReader<std::fs::File>> {
    /// Open a SOFT file from a path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Ok(Self::new(reader))
    }
}

impl SoftReader<std::io::BufReader<flate2::read::GzDecoder<std::fs::File>>> {
    /// Open a gzipped SOFT file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    pub fn open_gz<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let gz_reader = flate2::read::GzDecoder::new(file);
        let reader = std::io::BufReader::new(gz_reader);
        Ok(Self::new(reader))
    }
}

/// Represents a GDS (Dataset) record
#[derive(Debug, Clone)]
pub struct GdsRecord {
    pub geo_accession: String,
    pub title: String,
    pub description: String,
    pub platform: String,
    pub sample_organism: String,
    pub sample_type: String,
    pub feature_count: u32,
    pub sample_count: u32,
    pub subsets: Vec<GdsSubset>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
    pub column_descs: std::collections::HashMap<String, String>,
    pub data_table: Option<DataTable>,
}

/// Represents a GDS subset
#[derive(Debug, Clone)]
pub struct GdsSubset {
    pub local_id: String,
    pub description: String,
    pub sample_ids: Vec<String>,
    pub subset_type: String,
}

/// Represents a GSE (Series) record
#[derive(Debug, Clone)]
pub struct GseRecord {
    pub accession: String,
    pub title: String,
    pub summary: String,
    pub overall_design: String,
    pub submission_date: chrono::NaiveDate,
    pub sample_ids: Vec<String>,
    pub platform_ids: Vec<String>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
}

/// Represents a GSM (Sample) record
#[derive(Debug, Clone)]
pub struct GsmRecord {
    pub accession: String,
    pub title: String,
    pub characteristics: std::collections::HashMap<String, String>,
    pub platform_id: String,
    pub data_table: Option<DataTable>,
}

/// Represents a GPL (Platform) record
#[derive(Debug, Clone)]
pub struct GplRecord {
    pub accession: String,
    pub title: String,
    pub technology: String,
    pub annotation_table: Option<DataTable>,
}

/// Represents a data table
#[derive(Debug, Clone)]
pub struct DataTable {
    pub columns: Vec<ColumnDescriptor>,
    pub rows: Vec<Vec<String>>,
}

/// Column descriptor for data tables
#[derive(Debug, Clone)]
pub struct ColumnDescriptor {
    pub name: String,
    pub description: String,
}

impl GdsRecord {
    /// Convert to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn to_record_batch(&self) -> Result<RecordBatch> {
        use std::sync::Arc;

        use arrow::{
            array::{ArrayRef, Float64Array, StringArray},
            datatypes::{DataType, Field, Schema},
        };

        let table = self.data_table.as_ref().ok_or_else(|| {
            Error::InvalidFormat("No data table available for conversion".to_string())
        })?;

        if table.columns.is_empty() {
            return Err(Error::InvalidFormat("Empty data table".to_string()));
        }

        // Extract sample IDs from column names (skip first 2 columns: ID_REF,
        // IDENTIFIER)
        let sample_columns: Vec<&str> = table
            .columns
            .iter()
            .skip(2)
            .map(|col| col.name.as_str())
            .collect();

        // Create schema
        let mut fields = Vec::new();

        // ID_REF column
        fields.push(Field::new("id_ref", DataType::Utf8, false));

        // IDENTIFIER column
        fields.push(Field::new("identifier", DataType::Utf8, true));

        // Sample columns (Float64, nullable)
        for sample_id in &sample_columns {
            fields.push(Field::new(*sample_id, DataType::Float64, true));
        }

        let schema = Arc::new(Schema::new(fields));

        // Build arrays
        let mut arrays: Vec<ArrayRef> = Vec::new();

        // ID_REF array
        let id_ref_values: Vec<Option<&str>> = table
            .rows
            .iter()
            .map(|row| {
                row.first()
                    .and_then(|s| if s.is_empty() { None } else { Some(s.as_str()) })
            })
            .collect();
        arrays.push(Arc::new(StringArray::from(id_ref_values)));

        // IDENTIFIER array
        let identifier_values: Vec<Option<&str>> = table
            .rows
            .iter()
            .map(|row| {
                row.get(1)
                    .and_then(|s| if s.is_empty() { None } else { Some(s.as_str()) })
            })
            .collect();
        arrays.push(Arc::new(StringArray::from(identifier_values)));

        // Sample value arrays
        for col_idx in 2..table.columns.len() {
            let values: Vec<Option<f64>> = table
                .rows
                .iter()
                .map(|row| {
                    row.get(col_idx)
                        .filter(|s| !s.is_empty())
                        .and_then(|s| Self::parse_f64_nullable(s))
                })
                .collect();
            arrays.push(Arc::new(Float64Array::from(values)));
        }

        // Create RecordBatch
        let batch = RecordBatch::try_new(schema, arrays).map_err(Error::Arrow)?;

        Ok(batch)
    }

    /// Parse nullable float64 value with null sentinel handling
    fn parse_f64_nullable(s: &str) -> Option<f64> {
        match s.trim() {
            "" | "null" | "NULL" | "na" | "NA" | "n/a" | "N/A" | "nan" | "NaN" | "none"
            | "NONE" => None,
            _ => s.trim().parse::<f64>().ok(),
        }
    }
}

impl GseRecord {
    /// Convert to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn to_record_batch(&self) -> Result<RecordBatch> {
        // TODO: Implement Arrow conversion
        todo!("Implement Arrow conversion")
    }
}

impl GsmRecord {
    /// Convert to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn to_record_batch(&self) -> Result<RecordBatch> {
        // TODO: Implement Arrow conversion
        todo!("Implement Arrow conversion")
    }
}

impl GplRecord {
    /// Convert annotation to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn annotation_batch(&self) -> Result<RecordBatch> {
        // TODO: Implement Arrow conversion
        todo!("Implement Arrow conversion")
    }
}
