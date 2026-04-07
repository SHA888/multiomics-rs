//! SOFT format parser

use std::{collections::HashMap, io::BufRead, path::Path, sync::Arc};

use arrow::{
    array::{ArrayRef, Float64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};

use crate::{Error, Result};

// Constants for column indices
const ID_REF_COL: usize = 0;
const IDENTIFIER_COL: usize = 1;
const DUAL_VALUE_COL: usize = 2;
const DUAL_CH1_COL: usize = 3;
const DUAL_CH2_COL: usize = 4;

/// Parse a nullable float64 value with null sentinel handling
///
/// # Arguments
///
/// * `s` - String to parse
///
/// # Returns
///
/// * `Ok(None)` - Arrow null for recognized null sentinels
/// * `Ok(Some(f))` - Valid float value
/// * `Err` - Non-null, non-parseable string (surface to caller)
///
/// # Errors
///
/// Returns an error if the string is not a valid float and is not a recognized
/// null sentinel.
pub fn parse_f64_nullable(s: &str) -> Result<Option<f64>> {
    let trimmed = s.trim();

    // Check for null sentinels (case-insensitive)
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("null")
        || trimmed.eq_ignore_ascii_case("na")
        || trimmed.eq_ignore_ascii_case("n/a")
        || trimmed.eq_ignore_ascii_case("nan")
        || trimmed.eq_ignore_ascii_case("none")
    {
        return Ok(None);
    }

    // Try to parse as float
    match trimmed.parse::<f64>() {
        Ok(value) => Ok(Some(value)),
        Err(_) => Err(Error::InvalidFormat(format!("Invalid float value: '{s}'"))),
    }
}

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
    bom_stripped: bool,
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
            bom_stripped: false,
        }
    }

    /// Iterate over GSE records
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
                    // No series yet, continue parsing (implicit continue at end
                    // of loop)
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

        // Strip UTF-8 BOM on first line (avoid allocation by slicing)
        let line = if !self.bom_stripped && line.starts_with('\u{FEFF}') {
            self.bom_stripped = true;
            &line[3..] // BOM is 3 bytes in UTF-8
        } else {
            line.as_str()
        };

        // Normalize line endings and trim whitespace
        let line = line.trim_end_matches(['\r', '\n']).trim();

        // Skip blank lines silently in all states
        if line.is_empty() {
            return Ok(None);
        }

        // Handle different line types based on state
        match self.state {
            ParseState::Idle => Ok(self.handle_idle_state(line)),
            ParseState::InSeries => Ok(self.handle_series_state(line)),
            ParseState::InPlatform => Ok(self.handle_platform_state(line)),
            ParseState::InSample => self.handle_sample_state(line),
            ParseState::InDataset => Ok(self.handle_dataset_state(line)),
            ParseState::InSubset => Ok(self.handle_subset_state(line)),
            ParseState::InPlatformTable => self.handle_platform_table_state(line),
            ParseState::InSampleTable => self.handle_sample_table_state(line),
            ParseState::InDatasetTable => self.handle_dataset_table_state(line),
        }
    }

    /// Handle lines when in idle state
    fn handle_idle_state(&mut self, line: &str) -> Option<GseRecord> {
        if let Some(accession) = line.strip_prefix("^SERIES = ") {
            self.start_series(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^PLATFORM = ") {
            self.start_platform(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^SAMPLE = ") {
            self.start_sample(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^DATASET = ") {
            self.start_dataset(accession.trim())
        } else if let Some(accession) = line.strip_prefix("^SUBSET = ") {
            self.start_subset(accession.trim())
        } else {
            None
        }
    }

    /// Handle lines when in series state
    fn handle_series_state(&mut self, line: &str) -> Option<GseRecord> {
        if line.starts_with('^') {
            // Start of new section - return current series
            self.current_series.take()
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_series_metadata(key_value)
        } else {
            None
        }
    }

    /// Handle lines when in platform state
    fn handle_platform_state(&mut self, line: &str) -> Option<GseRecord> {
        if line.starts_with('^') {
            // Start of new section - return current series if any
            // Platform entity is consumed by series, not emitted directly
            self.current_platform = None;

            if line.strip_prefix("^PLATFORM = ").is_some() {
                // New platform - will be started by idle handler
                self.state = ParseState::Idle;
                self.current_series.take()
            } else if line.strip_prefix("^SERIES = ").is_some() {
                // New series - return current series if any
                self.state = ParseState::Idle;
                self.current_series.take()
            } else if line.strip_prefix("^SAMPLE = ").is_some() {
                // Sample start - continue in series context
                None
            } else {
                None
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_platform_metadata(key_value)
        } else {
            None
        }
    }

    /// Handle lines when in sample state
    fn handle_sample_state(&mut self, line: &str) -> Result<Option<GseRecord>> {
        if line.starts_with('^') {
            // Start of new section
            if line.strip_prefix("^SAMPLE = ").is_some() {
                // New sample - finish current one
                self.current_sample = None;
                Ok(None)
            } else if line.strip_prefix("^SERIES = ").is_some() {
                // New series - return current series
                self.current_sample = None;
                self.state = ParseState::Idle;
                Ok(self.current_series.take())
            } else {
                // Other entity - continue in series context
                self.current_sample = None;
                Ok(None)
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_sample_metadata(key_value)
        } else {
            Ok(None)
        }
    }

    /// Handle lines when in dataset state
    fn handle_dataset_state(&mut self, line: &str) -> Option<GseRecord> {
        if line.starts_with('^') {
            // Start of new section - finalize current dataset
            if line.strip_prefix("^DATASET = ").is_some() {
                // New dataset - current one is lost (no storage for multiple datasets)
                self.current_dataset = None;
                None
            } else {
                // Other entity - finalize current dataset
                self.current_dataset = None;
                None
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_dataset_metadata(key_value)
        } else {
            None
        }
    }

    /// Handle lines when in subset state
    fn handle_subset_state(&mut self, line: &str) -> Option<GseRecord> {
        if line.starts_with('^') {
            // Start of new section - add completed subset to dataset
            if let Some(subset) = self.current_subset.take() {
                if let Some(dataset) = &mut self.current_dataset {
                    dataset.subsets.push(subset);
                }
            }

            if line.strip_prefix("^SUBSET = ").is_some() {
                // New subset - start fresh
                self.start_subset(line.strip_prefix("^SUBSET = ").unwrap().trim())
            } else {
                // Other entity - return None for now
                None
            }
        } else if let Some(key_value) = line.strip_prefix('!') {
            self.parse_subset_metadata(key_value)
        } else {
            None
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
                    line.strip_prefix('#').and_then(|l| l.split_once('='))
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
                    line.strip_prefix('#').and_then(|l| l.split_once('='))
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
                    line.strip_prefix('#').and_then(|l| l.split_once('='))
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
            local_id: accession.to_string(),
            geo_accession: None,
            title: String::new(),
            summary: Vec::new(),
            overall_design: String::new(),
            series_type: Vec::new(),
            sample_ids: Vec::new(),
            contributor: Vec::new(),
            pubmed_id: Vec::new(),
            metadata: HashMap::new(),
        });
        None
    }

    /// Start parsing a new platform
    fn start_platform(&mut self, accession: &str) -> Option<GseRecord> {
        self.state = ParseState::InPlatform;
        self.current_platform = Some(GplRecord {
            local_id: accession.to_string(),
            geo_accession: None,
            title: String::new(),
            technology: String::new(),
            distribution: String::new(),
            organism: Vec::new(),
            manufacturer: String::new(),
            manufacture_protocol: Vec::new(),
            description: Vec::new(),
            contributor: Vec::new(),
            pubmed_id: Vec::new(),
            column_descs: HashMap::new(),
            metadata: HashMap::new(),
            annotation_table: None,
        });
        None
    }

    /// Start parsing a new sample
    fn start_sample(&mut self, accession: &str) -> Option<GseRecord> {
        self.state = ParseState::InSample;
        self.current_sample = Some(GsmRecord {
            local_id: accession.to_string(),
            geo_accession: None,
            title: String::new(),
            platform_id: String::new(),
            channel_count: 1, // Default to single channel
            source_name: Vec::new(),
            organism: Vec::new(),
            characteristics: Vec::new(),
            molecule: Vec::new(),
            label: Vec::new(),
            data_processing: Vec::new(),
            description: Vec::new(),
            metadata: HashMap::new(),
            data_table: None,
        });
        None
    }

    /// Start parsing a new dataset
    fn start_dataset(&mut self, accession: &str) -> Option<GseRecord> {
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
        None
    }

    /// Start parsing a new subset
    fn start_subset(&mut self, accession: &str) -> Option<GseRecord> {
        self.state = ParseState::InSubset;
        self.current_subset = Some(GdsSubset {
            local_id: accession.to_string(),
            description: String::new(),
            sample_ids: Vec::new(),
            subset_type: String::new(),
        });
        None
    }

    /// Parse series metadata
    fn parse_series_metadata(&mut self, key_value: &str) -> Option<GseRecord> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(series) = &mut self.current_series {
                match key {
                    "Series_title" => series.title = value.to_string(),
                    "Series_geo_accession" => series.geo_accession = Some(value.to_string()),
                    "Series_summary" => series.summary.push(value.to_string()),
                    "Series_overall_design" => series.overall_design = value.to_string(),
                    "Series_type" => series.series_type.push(value.to_string()),
                    "Series_sample_id" => series.sample_ids.push(value.to_string()),
                    "Series_contributor" => series.contributor.push(value.to_string()),
                    "Series_pubmed_id" => {
                        if let Ok(id) = value.parse::<u32>() {
                            series.pubmed_id.push(id);
                        }
                    }
                    _ => {
                        // Route all unrecognized attributes to metadata HashMap
                        // (includes download-only fields like _status, _submission_date, etc.)
                        series
                            .metadata
                            .entry(key.to_string())
                            .or_insert_with(Vec::new)
                            .push(value.to_string());
                    }
                }
            }
        }
        None
    }

    /// Parse platform metadata
    fn parse_platform_metadata(&mut self, key_value: &str) -> Option<GseRecord> {
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
        None
    }

    /// Parse sample metadata
    fn parse_sample_metadata(&mut self, key_value: &str) -> Result<Option<GseRecord>> {
        if let Some((key, value)) = key_value.split_once(" = ") {
            let key = key.trim();
            let value = value.trim();

            if let Some(sample) = &mut self.current_sample {
                match key {
                    "Sample_title" => sample.title = value.to_string(),
                    "Sample_geo_accession" => sample.geo_accession = Some(value.to_string()),
                    "Sample_platform_id" => sample.platform_id = value.to_string(),
                    "Sample_channel_count" => {
                        if let Ok(count) = value.parse::<u8>() {
                            sample.channel_count = count;
                        }
                    }
                    "!sample_table_begin" => {
                        self.state = ParseState::InSampleTable;
                        self.current_table = Some(DataTable {
                            columns: Vec::new(),
                            rows: Vec::new(),
                        });
                    }
                    key if key.starts_with("Sample_characteristics_") => {
                        // Extract channel and characteristics type
                        if let Some(char_key) = key.strip_prefix("Sample_characteristics_") {
                            // Parse channel number and characteristic type
                            if let Some((channel_str, char_type)) = char_key.split_once("_ch") {
                                // This is channel-specific: characteristics_ch1_disease
                                if let Ok(channel_num) = channel_str.parse::<usize>() {
                                    // Bounds check: channel count must fit in u8 (max 255)
                                    if channel_num > 254 {
                                        return Err(Error::InvalidFormat(format!(
                                            "Channel number {channel_num} exceeds maximum of 255"
                                        )));
                                    }
                                    // Update channel count if needed
                                    if let Ok(new_count) = u8::try_from(channel_num + 1) {
                                        if new_count > sample.channel_count {
                                            sample.channel_count = new_count;
                                        }
                                    }
                                    // Ensure we have enough channel slots
                                    while sample.characteristics.len() <= channel_num {
                                        sample.characteristics.push(HashMap::new());
                                    }
                                    sample.characteristics[channel_num]
                                        .insert(char_type.to_string(), value.to_string());
                                }
                            } else {
                                // Single channel - add to channel 0
                                if sample.characteristics.is_empty() {
                                    sample.characteristics.push(HashMap::new());
                                }
                                sample.characteristics[0]
                                    .insert(char_key.to_string(), value.to_string());
                            }
                        }
                    }
                    _ => {} // Handle other sample metadata as needed
                }
            }
        }
        Ok(None)
    }

    /// Parse dataset metadata
    fn parse_dataset_metadata(&mut self, key_value: &str) -> Option<GseRecord> {
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
        None
    }

    /// Parse subset metadata
    fn parse_subset_metadata(&mut self, key_value: &str) -> Option<GseRecord> {
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
        None
    }
}

/// Open a SOFT file from a path with auto-detect gzip
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn open_soft_file<P: AsRef<Path>>(
    path: P,
) -> Result<SoftReader<std::io::BufReader<std::fs::File>>> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    Ok(SoftReader::new(reader))
}

/// Open a gzipped SOFT file
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn open_soft_file_gz<P: AsRef<Path>>(
    path: P,
) -> Result<SoftReader<std::io::BufReader<flate2::read::GzDecoder<std::fs::File>>>> {
    let file = std::fs::File::open(path)?;
    let gz_reader = flate2::read::GzDecoder::new(file);
    let reader = std::io::BufReader::new(gz_reader);
    Ok(SoftReader::new(reader))
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
    pub local_id: String,
    pub geo_accession: Option<String>,
    pub title: String,
    pub summary: Vec<String>,
    pub overall_design: String,
    pub series_type: Vec<String>,
    pub sample_ids: Vec<String>,
    pub contributor: Vec<String>,
    pub pubmed_id: Vec<u32>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
}

/// Represents a GSM (Sample) record
#[derive(Debug, Clone)]
pub struct GsmRecord {
    pub local_id: String,
    pub geo_accession: Option<String>,
    pub title: String,
    pub platform_id: String,
    pub channel_count: u8,
    pub source_name: Vec<String>,
    pub organism: Vec<Vec<String>>,
    pub characteristics: Vec<std::collections::HashMap<String, String>>,
    pub molecule: Vec<String>,
    pub label: Vec<String>,
    pub data_processing: Vec<String>,
    pub description: Vec<String>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
    pub data_table: Option<DataTable>,
}

/// Represents a GPL (Platform) record
#[derive(Debug, Clone)]
pub struct GplRecord {
    pub local_id: String,
    pub geo_accession: Option<String>,
    pub title: String,
    pub technology: String,
    pub distribution: String,
    pub organism: Vec<String>,
    pub manufacturer: String,
    pub manufacture_protocol: Vec<String>,
    pub description: Vec<String>,
    pub contributor: Vec<String>,
    pub pubmed_id: Vec<u32>,
    pub column_descs: std::collections::HashMap<String, String>,
    pub metadata: std::collections::HashMap<String, Vec<String>>,
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

    /// Convert metadata to Arrow `RecordBatch` with attribute-value rows
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn metadata_batch(&self) -> Result<RecordBatch> {
        // Build schema: accession, key, value
        let fields = vec![
            Field::new("accession", DataType::Utf8, false),
            Field::new("key", DataType::Utf8, false),
            Field::new("value", DataType::Utf8, true),
        ];
        let schema = Schema::new(fields);

        // Collect all metadata as rows (multi-value fields produce multiple rows)
        let mut accessions: Vec<String> = Vec::new();
        let mut keys: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();

        // Add basic fields as metadata
        accessions.push(self.local_id.clone());
        keys.push("title".to_string());
        values.push(self.title.clone());

        if let Some(geo_accession) = &self.geo_accession {
            accessions.push(self.local_id.clone());
            keys.push("geo_accession".to_string());
            values.push(geo_accession.clone());
        }

        accessions.push(self.local_id.clone());
        keys.push("overall_design".to_string());
        values.push(self.overall_design.clone());

        // Add multi-value fields (each value gets its own row)
        for summary in &self.summary {
            accessions.push(self.local_id.clone());
            keys.push("summary".to_string());
            values.push(summary.clone());
        }

        for series_type in &self.series_type {
            accessions.push(self.local_id.clone());
            keys.push("series_type".to_string());
            values.push(series_type.clone());
        }

        for sample_id in &self.sample_ids {
            accessions.push(self.local_id.clone());
            keys.push("sample_id".to_string());
            values.push(sample_id.clone());
        }

        for contributor in &self.contributor {
            accessions.push(self.local_id.clone());
            keys.push("contributor".to_string());
            values.push(contributor.clone());
        }

        for pubmed_id in &self.pubmed_id {
            accessions.push(self.local_id.clone());
            keys.push("pubmed_id".to_string());
            values.push(pubmed_id.to_string());
        }

        // Add metadata HashMap entries (multi-value supported)
        for (key, values_list) in &self.metadata {
            for value in values_list {
                accessions.push(self.local_id.clone());
                keys.push(key.clone());
                values.push(value.clone());
            }
        }

        // Create Arrow arrays
        let arrays: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(accessions)),
            Arc::new(StringArray::from(keys)),
            Arc::new(StringArray::from(values)),
        ];

        // Add schema metadata (G1.2.5) before creating RecordBatch
        let mut metadata = HashMap::new();
        metadata.insert("geo_entity_type".to_string(), "series".to_string());
        metadata.insert("geo_local_id".to_string(), self.local_id.clone());
        if let Some(geo_accession) = &self.geo_accession {
            metadata.insert("geo_accession".to_string(), geo_accession.clone());
        }

        // Create schema with metadata
        let schema_with_metadata = schema.with_metadata(metadata);

        // Create RecordBatch once with metadata
        let batch =
            RecordBatch::try_new(Arc::new(schema_with_metadata), arrays).map_err(Error::Arrow)?;

        Ok(batch)
    }
}

impl GsmRecord {
    /// Convert to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    #[allow(clippy::too_many_lines)]
    pub fn to_record_batch(&self) -> Result<RecordBatch> {
        let Some(table) = &self.data_table else {
            return Err(Error::InvalidFormat("Sample has no data table".to_string()));
        };

        if table.rows.is_empty() {
            return Err(Error::InvalidFormat(
                "Sample data table has no rows".to_string(),
            ));
        }

        // Get column names (skip the first two which are ID_REF and IDENTIFIER)
        let value_columns: Vec<&str> = table
            .columns
            .iter()
            .skip(2)
            .map(|col| col.name.as_str())
            .collect();

        // Check if this is dual-channel (log ratio + ch1/ch2 values) or single-channel
        // Dual-channel pattern: VALUE, CH1, CH2 (case-insensitive)
        let is_dual_channel = value_columns.len() >= 3
            && value_columns[0].to_ascii_uppercase().contains("VALUE")
            && value_columns[1].to_ascii_uppercase().contains("CH1")
            && value_columns[2].to_ascii_uppercase().contains("CH2");

        // Build schema
        let mut fields = vec![
            Field::new("id_ref", DataType::Utf8, false),
            Field::new("identifier", DataType::Utf8, true),
        ];

        if is_dual_channel {
            fields.push(Field::new("value", DataType::Float64, true)); // log ratio
            fields.push(Field::new("ch1_value", DataType::Float64, true));
            fields.push(Field::new("ch2_value", DataType::Float64, true));
        } else {
            // Single channel - just one value column per sample
            for col_name in &value_columns {
                fields.push(Field::new(
                    col_name.to_ascii_lowercase(),
                    DataType::Float64,
                    true,
                ));
            }
        }

        // Add auxiliary columns as Utf8 (caller can cast downstream)
        for col in table.columns.iter().skip(
            2 + if is_dual_channel {
                3
            } else {
                value_columns.len()
            },
        ) {
            fields.push(Field::new(
                col.name.to_ascii_lowercase(),
                DataType::Utf8,
                true,
            ));
        }

        let schema = Schema::new(fields);

        // Build arrays
        let mut id_refs: Vec<String> = Vec::new();
        let mut identifiers: Vec<Option<String>> = Vec::new();
        let mut value_arrays: Vec<Vec<Option<f64>>> = if is_dual_channel {
            vec![Vec::new(), Vec::new(), Vec::new()] // value, ch1, ch2
        } else {
            vec![Vec::new(); value_columns.len()]
        };
        let mut auxiliary_arrays: Vec<Vec<String>> = Vec::new();

        // Initialize auxiliary arrays
        for _ in table.columns.iter().skip(
            2 + if is_dual_channel {
                3
            } else {
                value_columns.len()
            },
        ) {
            auxiliary_arrays.push(Vec::new());
        }

        // Process rows
        for row in &table.rows {
            if row.len() < 2 {
                continue;
            }

            id_refs.push(row[ID_REF_COL].clone());
            identifiers.push(row.get(IDENTIFIER_COL).cloned());

            if is_dual_channel {
                // Parse value columns: log ratio, ch1, ch2
                for (i, col_idx) in [DUAL_VALUE_COL, DUAL_CH1_COL, DUAL_CH2_COL]
                    .iter()
                    .enumerate()
                {
                    let value = row.get(*col_idx).cloned().unwrap_or_default();
                    let parsed = parse_f64_nullable(&value)?;
                    value_arrays[i].push(parsed);
                }

                // Handle auxiliary columns
                for (i, aux_col_idx) in (DUAL_CH2_COL + 1..row.len()).enumerate() {
                    if let Some(aux_array) = auxiliary_arrays.get_mut(i) {
                        aux_array.push(row[aux_col_idx].clone());
                    }
                }
            } else {
                // Single channel - parse each value column
                for (i, col_idx) in (2..row.len()).enumerate() {
                    if i < value_arrays.len() {
                        let value = row.get(col_idx).cloned().unwrap_or_default();
                        let parsed = parse_f64_nullable(&value)?;
                        value_arrays[i].push(parsed);
                    } else {
                        // Auxiliary columns - use safe index math
                        let aux_index = i.saturating_sub(value_arrays.len());
                        if let Some(aux_array) = auxiliary_arrays.get_mut(aux_index) {
                            aux_array.push(row[col_idx].clone());
                        }
                    }
                }
            }
        }

        // Create Arrow arrays
        let mut arrays: Vec<ArrayRef> = vec![
            Arc::new(StringArray::from(id_refs)),
            Arc::new(StringArray::from(identifiers)),
        ];

        // Add value arrays
        for values in value_arrays {
            arrays.push(Arc::new(Float64Array::from(values)));
        }

        // Add auxiliary arrays
        for aux_values in auxiliary_arrays {
            arrays.push(Arc::new(StringArray::from(aux_values)));
        }

        // Add schema metadata (G1.2.5) before creating RecordBatch
        let mut metadata = HashMap::new();
        metadata.insert(
            "geo_channel_count".to_string(),
            self.channel_count.to_string(),
        );
        if let Some(geo_accession) = &self.geo_accession {
            metadata.insert("geo_accession".to_string(), geo_accession.clone());
        }
        metadata.insert("geo_platform_id".to_string(), self.platform_id.clone());

        // Create schema with metadata
        let schema_with_metadata = schema.with_metadata(metadata);

        // Create RecordBatch once with metadata
        let batch =
            RecordBatch::try_new(Arc::new(schema_with_metadata), arrays).map_err(Error::Arrow)?;

        Ok(batch)
    }
}

impl GplRecord {
    /// Convert annotation to Arrow `RecordBatch`
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails.
    pub fn annotation_batch(&self) -> Result<RecordBatch> {
        let Some(table) = &self.annotation_table else {
            return Err(Error::InvalidFormat(
                "Platform has no annotation table".to_string(),
            ));
        };

        if table.rows.is_empty() {
            return Err(Error::InvalidFormat(
                "Platform annotation table has no rows".to_string(),
            ));
        }

        // Standard platform headers mapping (per GEO spec to snake_case)
        let standard_headers = HashMap::from([
            ("ID", "id"),
            ("ID_REF", "id_ref"),
            ("IDENTIFIER", "identifier"),
            ("SEQUENCE", "sequence"),
            ("GB_ACC", "gb_acc"),
            ("GENE_SYMBOL", "gene_symbol"),
            ("ENTREZ_ID", "entrez_id"),
            ("DESCRIPTION", "description"),
            ("SPOT_ID", "spot_id"),
            ("DEFINITION", "definition"),
            ("ONTOLOGY", "ontology"),
            ("SYNONYM", "synonym"),
            ("GO_ID", "go_id"),
            ("GO_TERM", "go_term"),
            ("PROBE_ID", "probe_id"),
            ("ACCESSION", "accession"),
        ]);

        // Build schema with standard headers and pass-through for non-standard
        let mut fields = Vec::new();
        let mut column_mappings: Vec<(String, String)> = Vec::new(); // (original, snake_case)

        for col in &table.columns {
            let snake_name = standard_headers.get(&col.name.as_str()).map_or_else(
                || col.name.to_ascii_lowercase(),
                std::string::ToString::to_string,
            );

            column_mappings.push((col.name.clone(), snake_name.clone()));

            // Add field metadata if available
            let mut field = Field::new(snake_name, DataType::Utf8, true);
            if let Some(description) = self.column_descs.get(&col.name) {
                let metadata = HashMap::from([("geo_col_desc".to_string(), description.clone())]);
                field = field.with_metadata(metadata);
            }
            fields.push(field);
        }

        let schema = Schema::new(fields);

        // Build arrays
        let mut arrays: Vec<Vec<String>> = vec![Vec::new(); table.columns.len()];

        // Process rows
        for row in &table.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < arrays.len() {
                    arrays[i].push(cell.clone());
                }
            }
        }

        // Create Arrow arrays
        let mut arrow_arrays: Vec<ArrayRef> = Vec::new();

        for (i, (_original_name, _snake_name)) in column_mappings.iter().enumerate() {
            if i < arrays.len() {
                let array = Arc::new(StringArray::from(arrays[i].clone()));
                arrow_arrays.push(array);
            }
        }

        // Add schema metadata (G1.2.5) before creating RecordBatch
        let mut metadata = HashMap::new();
        metadata.insert("geo_entity_type".to_string(), "platform".to_string());
        if let Some(geo_accession) = &self.geo_accession {
            metadata.insert("geo_accession".to_string(), geo_accession.clone());
        }
        metadata.insert("geo_local_id".to_string(), self.local_id.clone());

        // Create schema with metadata
        let schema_with_metadata = schema.with_metadata(metadata);

        // Create RecordBatch once with metadata
        let batch = RecordBatch::try_new(Arc::new(schema_with_metadata), arrow_arrays)
            .map_err(Error::Arrow)?;

        Ok(batch)
    }
}
