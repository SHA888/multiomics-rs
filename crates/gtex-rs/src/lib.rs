//! # gtex-rs
//!
//! `GTEx` GCT format reader → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use gtex_rs::GtexReader;
//!
//! let mut reader = GtexReader::from_gct("GTEx_Analysis_v8_gene_median_tpm.gct.gz")?;
//! let batch = reader.median_tpm()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod error;
pub mod reader;
pub mod tissue;

pub use error::{Error, Result};
pub use reader::GtexReader;
pub use tissue::TissueExpression;
