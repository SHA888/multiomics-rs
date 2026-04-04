//! # string-rs
//!
//! STRING protein interaction TSV reader → Arrow `RecordBatches`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use string_rs::ProteinInteractionReader;
//!
//! let mut reader = ProteinInteractionReader::from_tsv("9606.protein.links.full.v12.0.txt.gz")?;
//! let interactions = reader.read_interactions()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)]

pub mod error;
pub mod info;
pub mod interaction;

pub use error::{Error, Result};
pub use info::ProteinInfo;
pub use interaction::{Direction, ProteinInteraction, ProteinInteractionReader};
