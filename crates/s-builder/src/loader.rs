use anyhow::Result;

use std::fs;

use std::path::Path;

use crate::metadata::DatasetMetadata;

pub fn load_metadata<P: AsRef<Path>>(path: P) -> Result<DatasetMetadata> {
    let text = fs::read_to_string(path)?;

    let metadata = serde_json::from_str(&text)?;

    Ok(metadata)
}
