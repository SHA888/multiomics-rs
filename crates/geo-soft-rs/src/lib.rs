//! # geo-soft-rs
//!
//! Parser for NCBI GEO SOFT format → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use geo_soft_rs::SoftReader;
//!
//! let mut reader = SoftReader::open("GSE65682_family.soft.gz")?;
//! for record in reader.series() {
//!     let gse = record?;
//!     println!("{}: {} samples", gse.accession, gse.sample_ids.len());
//!     // TODO: Get expression matrix when implemented
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod error;
pub mod parser;

pub use error::{Error, Result};
pub use parser::{GplRecord, GseRecord, GsmRecord, SoftReader};
