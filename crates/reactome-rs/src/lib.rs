//! # reactome-rs
//!
//! Reactome pathway TSV reader → Arrow RecordBatches.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use reactome_rs::GenePathwayReader;
//!
//! let mut reader = GenePathwayReader::from_tsv("pathways.tsv")?;
//! let pathways = reader.read_pathways()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod hierarchy;
pub mod pathway;

pub use error::{Error, Result};
pub use hierarchy::{PathwayHierarchy, PathwayHierarchyReader};
pub use pathway::{GenePathway, GenePathwayReader};
