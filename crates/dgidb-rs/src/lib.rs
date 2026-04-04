//! # dgidb-rs
//!
//! `DGIdb` drug-gene interaction TSV reader → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use dgidb_rs::InteractionReader;
//!
//! let mut reader = InteractionReader::from_tsv("interactions.tsv")?;
//! let interactions = reader.read_interactions()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod error;
pub mod gene;
pub mod interaction;

pub use error::{Error, Result};
pub use gene::DruggableGene;
pub use interaction::{DrugGeneInteraction, InteractionReader};
