//! SOF package manifest.

use serde::{Deserialize, Serialize};

/// SOF package description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SofManifest {
    /// Format version.
    pub version: String,

    /// Dataset name.
    pub name: String,

    /// Description.
    pub description: String,

    /// Available datasets.
    pub datasets: Vec<String>,
}

impl SofManifest {
    /// Creates a new manifest.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: "0.1.0".to_string(),
            name: name.into(),
            description: String::new(),
            datasets: Vec::new(),
        }
    }

    /// Adds a dataset entry.
    pub fn add_dataset(&mut self, name: impl Into<String>) {
        self.datasets.push(name.into());
    }
}
