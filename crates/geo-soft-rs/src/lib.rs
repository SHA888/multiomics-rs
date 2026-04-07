//! # geo-soft-rs
//!
//! Parser for NCBI GEO SOFT format → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use geo_soft_rs::open_soft_file_gz;
//!
//! let mut reader = open_soft_file_gz("GSE65682_family.soft.gz")?;
//! for record in reader.series() {
//!     let gse = record?;
//!     println!("{}: {} samples", gse.local_id, gse.sample_ids.len());
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
pub use parser::{
    GdsRecord, GdsSubset, GplRecord, GseRecord, GsmRecord, SoftReader, open_soft_file,
    open_soft_file_gz, parse_f64_nullable,
};
