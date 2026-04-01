//! # uniprot-rs
//!
//! UniProt Swiss-Prot TSV/XML reader → Arrow RecordBatches.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use uniprot_rs::ProteinReader;
//!
//! let mut reader = ProteinReader::from_tsv("uniprot_sprot.tsv")?;
//! let annotations = reader.read_annotations()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod protein;

pub use error::{Error, Result};
pub use protein::{ProteinAnnotation, ProteinReader};
