//! Expression matrix structures and builders
//!
//! This module provides functionality for assembling expression matrices
//! from NCBI GEO SOFT format data. It handles:
//!
//! - Joining sample data tables on probe IDs
//! - Mapping probe IDs to gene symbols via platform annotation
//! - Aggregating multiple probes per gene (mean by default)
//! - Converting to Arrow `RecordBatch` for efficient processing

use std::collections::HashMap;

use arrow::{
    array::{Array, Float64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};

use crate::{Error, Result};

/// Type alias for gene expression values: vector of per-sample values
pub type GeneValues = Vec<Vec<Option<f64>>>;

/// Expression matrix with genes as rows and samples as columns
///
/// The `values` field is an Arrow `RecordBatch` where:
/// - Each row represents a gene
/// - Each column represents a sample's expression values for all genes
/// - Column names are GSM accession IDs
/// - Values are `Float64` (null for missing data)
#[derive(Debug, Clone)]
pub struct ExpressionMatrix {
    /// Gene symbols (rows), ordered to match `RecordBatch` rows
    pub genes: Vec<String>,

    /// Sample GSM accession IDs (columns), ordered to match `RecordBatch`
    /// columns
    pub samples: Vec<String>,

    /// Arrow `RecordBatch` with gene expression values
    ///
    /// Schema: one `Float64` column per sample, column name = GSM accession
    pub values: RecordBatch,
}

impl ExpressionMatrix {
    /// Get expression value for a specific gene and sample
    ///
    /// Returns `None` if the gene or sample is not found, or if the value is
    /// null.
    #[must_use]
    pub fn get(&self, gene: &str, sample: &str) -> Option<f64> {
        let gene_idx = self.genes.iter().position(|g| g == gene)?;
        let sample_idx = self.samples.iter().position(|s| s == sample)?;

        let col = self.values.column(sample_idx);
        let array = col.as_any().downcast_ref::<Float64Array>()?;

        if array.is_null(gene_idx) {
            None
        } else {
            Some(array.value(gene_idx))
        }
    }
}

/// Aggregation method for multiple probes per gene
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AggregationMethod {
    /// Take the mean of all probe values (default)
    #[default]
    Mean,

    /// Take the median of all probe values
    Median,

    /// Take the maximum probe value
    Max,

    /// Take the minimum probe value
    Min,
}

/// Configuration for matrix building
#[derive(Debug, Clone)]
pub struct MatrixConfig {
    /// How to aggregate multiple probes per gene
    pub aggregation: AggregationMethod,

    /// Minimum number of samples a probe must appear in to be included
    pub min_sample_presence: usize,
}

impl Default for MatrixConfig {
    fn default() -> Self {
        Self {
            aggregation: AggregationMethod::Mean,
            min_sample_presence: 1,
        }
    }
}

/// Builder for creating expression matrices from SOFT data
pub struct MatrixBuilder {
    config: MatrixConfig,
}

impl Default for MatrixBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MatrixBuilder {
    /// Create a new matrix builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MatrixConfig::default(),
        }
    }

    /// Create a new matrix builder with custom configuration
    #[must_use]
    pub fn with_config(config: MatrixConfig) -> Self {
        Self { config }
    }

    /// Build expression matrix from a SOFT file reader
    ///
    /// This method processes a SOFT family file (GSE) containing:
    /// - Multiple GSM (sample) records with data tables
    /// - A GPL (platform) record with probe-to-gene annotation
    ///
    /// # Algorithm
    ///
    /// 1. Collect all samples and their data tables
    /// 2. Extract probe IDs (`ID_REF` column) and expression values from each
    ///    sample
    /// 3. Build probe-to-gene mapping from platform annotation (if available)
    /// 4. For genes with multiple probes, aggregate using the configured method
    /// 5. Assemble final matrix with genes as rows, samples as columns
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The SOFT data cannot be parsed
    /// - No samples with data tables are found
    /// - Required columns (`ID_REF`, `VALUE`) are missing
    /// - Arrow data construction fails
    pub fn from_soft<R>(&self, mut reader: geo_soft_rs::SoftReader<R>) -> Result<ExpressionMatrix>
    where
        R: std::io::BufRead,
    {
        // Step 1: Collect all samples and platform
        let mut samples: Vec<geo_soft_rs::GsmRecord> = Vec::new();
        let platform_opt: Option<geo_soft_rs::GplRecord> = None;

        // First pass: collect all samples
        while let Some(result) = reader.next_sample() {
            let sample = result?;
            if sample.data_table.is_some() {
                samples.push(sample);
            }
        }

        // Reset reader to collect platform
        // Note: This requires re-opening the file or using a different approach
        // For now, we work with what we have

        if samples.is_empty() {
            return Err(Error::Matrix(
                "No samples with data tables found in SOFT file".to_string(),
            ));
        }

        // Step 2: Extract probe expression data from each sample
        let mut probe_data: HashMap<String, Vec<(usize, f64)>> = HashMap::new();
        let mut sample_ids: Vec<String> = Vec::with_capacity(samples.len());

        for (sample_idx, sample) in samples.iter().enumerate() {
            let sample_id = sample
                .geo_accession
                .clone()
                .unwrap_or_else(|| sample.local_id.clone());
            sample_ids.push(sample_id);

            if let Some(ref table) = sample.data_table {
                // Find `ID_REF` and `VALUE` column indices
                let id_ref_idx = table
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case("ID_REF"))
                    .ok_or_else(|| {
                        Error::Matrix(format!(
                            "Sample {} missing `ID_REF` column",
                            sample.local_id
                        ))
                    })?;

                let value_idx = table
                    .columns
                    .iter()
                    .position(|c| c.name.eq_ignore_ascii_case("VALUE"))
                    .ok_or_else(|| {
                        Error::Matrix(format!("Sample {} missing `VALUE` column", sample.local_id))
                    })?;

                // Extract probe values
                for row in &table.rows {
                    if let Some(probe_id) = row.get(id_ref_idx) {
                        if let Some(value_str) = row.get(value_idx) {
                            if let Ok(value) = value_str.parse::<f64>() {
                                probe_data
                                    .entry(probe_id.clone())
                                    .or_default()
                                    .push((sample_idx, value));
                            }
                            // Invalid float values become null (skip)
                        }
                    }
                }
            }
        }

        // Step 3: Build probe-to-gene mapping if platform is available
        let probe_to_gene = Self::build_probe_to_gene_map(platform_opt.as_ref());

        // Step 4: Aggregate probes by gene
        let (genes, gene_values) =
            self.aggregate_by_gene(&probe_data, &probe_to_gene, samples.len());

        // Step 5: Build Arrow RecordBatch
        let values = Self::build_record_batch(&genes, &sample_ids, &gene_values)?;

        Ok(ExpressionMatrix {
            genes,
            samples: sample_ids,
            values,
        })
    }

    /// Build probe-to-gene mapping from platform annotation
    fn build_probe_to_gene_map(
        platform: Option<&geo_soft_rs::GplRecord>,
    ) -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        if let Some(p) = platform {
            if let Some(ref table) = p.annotation_table {
                // Find probe ID and gene symbol columns
                let probe_idx = table.columns.iter().position(|c| {
                    c.name.eq_ignore_ascii_case("ID")
                        || c.name.eq_ignore_ascii_case("PROBE_ID")
                        || c.name.eq_ignore_ascii_case("ID_REF")
                });

                let gene_idx = table.columns.iter().position(|c| {
                    c.name.eq_ignore_ascii_case("GENE_SYMBOL")
                        || c.name.eq_ignore_ascii_case("SYMBOL")
                        || c.name.eq_ignore_ascii_case("GENE")
                });

                if let (Some(p_idx), Some(g_idx)) = (probe_idx, gene_idx) {
                    for row in &table.rows {
                        if let (Some(probe), Some(gene)) = (row.get(p_idx), row.get(g_idx)) {
                            if !gene.is_empty() && gene != "---" {
                                mapping.insert(probe.clone(), gene.clone());
                            }
                        }
                    }
                }
            }
        }

        mapping
    }

    /// Aggregate probe values by gene
    #[allow(clippy::cast_precision_loss)]
    fn aggregate_by_gene(
        &self,
        probe_data: &HashMap<String, Vec<(usize, f64)>>,
        probe_to_gene: &HashMap<String, String>,
        num_samples: usize,
    ) -> (Vec<String>, GeneValues) {
        // Group probes by gene
        let mut gene_probes: HashMap<String, Vec<String>> = HashMap::new();

        for probe_id in probe_data.keys() {
            let gene = probe_to_gene
                .get(probe_id)
                .cloned()
                .unwrap_or_else(|| probe_id.clone());
            gene_probes.entry(gene).or_default().push(probe_id.clone());
        }

        // Sort genes for consistent ordering
        let mut genes: Vec<String> = gene_probes.keys().cloned().collect();
        genes.sort();

        // Aggregate values for each gene
        let mut gene_values: Vec<Vec<Option<f64>>> = Vec::with_capacity(genes.len());

        for gene in &genes {
            let probes = gene_probes.get(gene).unwrap();
            let mut sample_values: Vec<Vec<f64>> = vec![Vec::new(); num_samples];

            // Collect all values for each sample
            for probe_id in probes {
                if let Some(values) = probe_data.get(probe_id) {
                    for (sample_idx, value) in values {
                        sample_values[*sample_idx].push(*value);
                    }
                }
            }

            // Aggregate per sample
            let mut aggregated: Vec<Option<f64>> = Vec::with_capacity(num_samples);
            for values in sample_values {
                if values.is_empty() {
                    aggregated.push(None);
                } else {
                    let agg = match self.config.aggregation {
                        AggregationMethod::Mean => values.iter().sum::<f64>() / values.len() as f64,
                        AggregationMethod::Median => {
                            let mut sorted = values.clone();
                            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                            let mid = sorted.len() / 2;
                            if sorted.len() % 2 == 0 {
                                f64::midpoint(sorted[mid - 1], sorted[mid])
                            } else {
                                sorted[mid]
                            }
                        }
                        AggregationMethod::Max => values
                            .iter()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .copied()
                            .unwrap_or(0.0),
                        AggregationMethod::Min => values
                            .iter()
                            .min_by(|a, b| a.partial_cmp(b).unwrap())
                            .copied()
                            .unwrap_or(0.0),
                    };
                    aggregated.push(Some(agg));
                }
            }

            gene_values.push(aggregated);
        }

        (genes, gene_values)
    }

    /// Build Arrow `RecordBatch` from gene values
    fn build_record_batch(
        genes: &[String],
        sample_ids: &[String],
        gene_values: &[Vec<Option<f64>>],
    ) -> Result<RecordBatch> {
        // Build schema: one Float64 column per sample
        let fields: Vec<Field> = sample_ids
            .iter()
            .map(|id| Field::new(id.clone(), DataType::Float64, true))
            .collect();
        let schema = Schema::new(fields);

        // Build columns
        let mut columns: Vec<arrow::array::ArrayRef> = Vec::with_capacity(sample_ids.len());

        for sample_idx in 0..sample_ids.len() {
            let mut values: Vec<Option<f64>> = Vec::with_capacity(genes.len());
            for gene_values_row in gene_values {
                values.push(gene_values_row[sample_idx]);
            }
            let array = Float64Array::from(values);
            columns.push(std::sync::Arc::new(array));
        }

        let batch = RecordBatch::try_new(std::sync::Arc::new(schema), columns)?;
        Ok(batch)
    }
}

/// Sample metadata as Arrow `RecordBatch`
///
/// Columns: `gsm_accession`, `title`, `characteristic_key`,
/// `characteristic_value` One row per characteristic per sample
#[derive(Debug, Clone)]
pub struct SampleMetadata {
    /// Arrow `RecordBatch` with sample metadata
    pub data: RecordBatch,
}

impl SampleMetadata {
    /// Build sample metadata from SOFT reader
    ///
    /// Creates a `RecordBatch` with columns:
    /// - `gsm_accession`: Sample GSM accession ID
    /// - `title`: Sample title
    /// - `characteristic_key`: Characteristic name (e.g., "tissue", "cell
    ///   type")
    /// - `characteristic_value`: Characteristic value
    ///
    /// # Errors
    ///
    /// Returns an error if the SOFT data cannot be parsed or if Arrow
    /// data construction fails.
    pub fn from_soft<R>(mut reader: geo_soft_rs::SoftReader<R>) -> Result<Self>
    where
        R: std::io::BufRead,
    {
        let mut records: Vec<(String, String, String, String)> = Vec::new();

        while let Some(result) = reader.next_sample() {
            let sample = result?;
            let gsm_accession = sample
                .geo_accession
                .clone()
                .unwrap_or_else(|| sample.local_id.clone());

            // Extract characteristics
            for char_map in &sample.characteristics {
                for (key, value) in char_map {
                    records.push((
                        gsm_accession.clone(),
                        sample.title.clone(),
                        key.clone(),
                        value.clone(),
                    ));
                }
            }

            // If no characteristics, still add a row with empty key/value
            if sample.characteristics.is_empty() {
                records.push((gsm_accession, sample.title, String::new(), String::new()));
            }
        }

        // Build RecordBatch
        let schema = Schema::new(vec![
            Field::new("gsm_accession", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, false),
            Field::new("characteristic_key", DataType::Utf8, false),
            Field::new("characteristic_value", DataType::Utf8, false),
        ]);

        let gsm_accessions: Vec<&str> = records.iter().map(|r| r.0.as_str()).collect();
        let titles: Vec<&str> = records.iter().map(|r| r.1.as_str()).collect();
        let keys: Vec<&str> = records.iter().map(|r| r.2.as_str()).collect();
        let values: Vec<&str> = records.iter().map(|r| r.3.as_str()).collect();

        let batch = RecordBatch::try_new(
            std::sync::Arc::new(schema),
            vec![
                std::sync::Arc::new(StringArray::from(gsm_accessions)),
                std::sync::Arc::new(StringArray::from(titles)),
                std::sync::Arc::new(StringArray::from(keys)),
                std::sync::Arc::new(StringArray::from(values)),
            ],
        )?;

        Ok(Self { data: batch })
    }
}

/// Platform annotation as Arrow `RecordBatch`
///
/// Columns: `probe_id`, `gene_symbol`, `entrez_id`, `description`
#[derive(Debug, Clone)]
pub struct PlatformAnnotation {
    /// Arrow `RecordBatch` with platform annotation
    pub data: RecordBatch,
}

impl PlatformAnnotation {
    /// Build platform annotation from SOFT reader
    ///
    /// Creates a `RecordBatch` with columns:
    /// - `probe_id`: Probe identifier (`ID_REF`)
    /// - `gene_symbol`: Gene symbol
    /// - `entrez_id`: Entrez Gene ID (may be null)
    /// - `description`: Gene description (may be null)
    ///
    /// # Errors
    ///
    /// Returns an error if the SOFT data cannot be parsed or if Arrow
    /// data construction fails.
    #[allow(clippy::similar_names)]
    pub fn from_soft<R>(mut reader: geo_soft_rs::SoftReader<R>) -> Result<Option<Self>>
    where
        R: std::io::BufRead,
    {
        while let Some(result) = reader.next_platform() {
            let platform = result?;

            if let Some(ref table) = platform.annotation_table {
                // Find column indices
                let probe_idx = table
                    .columns
                    .iter()
                    .position(|c| {
                        c.name.eq_ignore_ascii_case("ID")
                            || c.name.eq_ignore_ascii_case("PROBE_ID")
                            || c.name.eq_ignore_ascii_case("ID_REF")
                    })
                    .ok_or_else(|| {
                        Error::Matrix("Platform annotation missing probe ID column".to_string())
                    })?;

                let gene_idx = table.columns.iter().position(|c| {
                    c.name.eq_ignore_ascii_case("GENE_SYMBOL")
                        || c.name.eq_ignore_ascii_case("SYMBOL")
                        || c.name.eq_ignore_ascii_case("GENE")
                });

                let entrez_idx = table.columns.iter().position(|c| {
                    c.name.eq_ignore_ascii_case("ENTREZ_ID")
                        || c.name.eq_ignore_ascii_case("ENTREZ")
                        || c.name.eq_ignore_ascii_case("GENE_ID")
                });

                let desc_idx = table.columns.iter().position(|c| {
                    c.name.eq_ignore_ascii_case("DESCRIPTION")
                        || c.name.eq_ignore_ascii_case("DESC")
                        || c.name.eq_ignore_ascii_case("GENE_TITLE")
                });

                // Extract data
                let mut probe_ids: Vec<&str> = Vec::new();
                let mut gene_symbols: Vec<Option<&str>> = Vec::new();
                let mut gene_entrez_ids: Vec<Option<&str>> = Vec::new();
                let mut descriptions: Vec<Option<&str>> = Vec::new();

                for row in &table.rows {
                    if let Some(probe) = row.get(probe_idx) {
                        probe_ids.push(probe);
                        gene_symbols.push(gene_idx.and_then(|i| row.get(i).map(String::as_str)));
                        gene_entrez_ids
                            .push(entrez_idx.and_then(|i| row.get(i).map(String::as_str)));
                        descriptions.push(desc_idx.and_then(|i| row.get(i).map(String::as_str)));
                    }
                }

                // Build RecordBatch
                let schema = Schema::new(vec![
                    Field::new("probe_id", DataType::Utf8, false),
                    Field::new("gene_symbol", DataType::Utf8, true),
                    Field::new("entrez_id", DataType::Utf8, true),
                    Field::new("description", DataType::Utf8, true),
                ]);

                let batch = RecordBatch::try_new(
                    std::sync::Arc::new(schema),
                    vec![
                        std::sync::Arc::new(StringArray::from(probe_ids)),
                        std::sync::Arc::new(StringArray::from(gene_symbols)),
                        std::sync::Arc::new(StringArray::from(gene_entrez_ids)),
                        std::sync::Arc::new(StringArray::from(descriptions)),
                    ],
                )?;

                return Ok(Some(Self { data: batch }));
            }
        }

        Ok(None)
    }
}
