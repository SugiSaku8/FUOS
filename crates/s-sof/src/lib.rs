#![warn(missing_docs)]

//! Suruga Ocean Format (SOF)
//!
//! This crate provides reading and writing support
//! for SOS ocean datasets.

pub mod boundary;
pub mod builder;
pub mod dataset;
pub mod error;
pub mod forcing;
pub mod format;
pub mod grid;
pub mod initial;
pub mod manifest;
pub mod package;
pub mod package_reader;
pub mod reader;
pub mod restart;
pub mod velocity;
pub mod writer;

pub use initial::{InitialReader, InitialWriter};

pub use restart::{RestartReader, RestartWriter};

pub use forcing::{ForcingData, ForcingReader, ForcingWriter};
pub use velocity::{VelocityReader, VelocityWriter};
// -----------------------------------------------------------------------------
// Re-exports
// -----------------------------------------------------------------------------

pub use boundary::{BoundaryReader, BoundaryWriter};

pub use dataset::SofDataset;

pub use error::{Result, SofError};
