//! # transcriptomic-rs
//!
//! Expression matrix assembly and normalization → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use geo_soft_rs::SoftReader;
//! use transcriptomic_rs::MatrixBuilder;
//!
//! let reader = SoftReader::open("GSE65682_family.soft.gz")?;
//! let matrix = MatrixBuilder::new().from_soft(reader)?;
//! let normalized = transcriptomic_rs::Normalize::log2(&matrix);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod error;
pub mod matrix;
pub mod normalization;

pub use error::{Error, Result};
pub use matrix::{
    AggregationMethod, ExpressionMatrix, GeneValues, MatrixBuilder, MatrixConfig,
    PlatformAnnotation, SampleMetadata,
};
pub use normalization::Normalize;
