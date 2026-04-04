//! # open-targets-rs
//!
//! Open Targets Platform Parquet reader → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use open_targets_rs::EvidenceReader;
//!
//! let mut reader = EvidenceReader::from_parquet("path/to/evidence/")?;
//! let batches = reader.read_batches()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod drug;
pub mod error;
pub mod evidence;
pub mod target;

pub use drug::{DrugMechanism, DrugReader};
pub use error::{Error, Result};
pub use evidence::{EvidenceReader, TargetEvidence};
pub use target::{TargetAnnotation, TargetReader};
