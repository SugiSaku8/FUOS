#![warn(missing_docs)]

//! Suruga Ocean Format (SOF)
//!
//! This crate provides reading and writing support
//! for SOS ocean datasets.

pub mod boundary;
pub mod builder;
pub mod dataset;
pub mod error;
pub mod format;
pub mod grid;
pub mod manifest;
pub mod package;
pub mod package_reader;
pub mod reader;
pub mod writer;
// -----------------------------------------------------------------------------
// Re-exports
// -----------------------------------------------------------------------------

pub use boundary::{BoundaryReader, BoundaryWriter};

pub use dataset::SofDataset;

pub use error::{Result, SofError};
