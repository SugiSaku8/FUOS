use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,

    pub version: String,

    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub name: String,

    pub offset: u64,

    pub size: u64,

    pub element_count: usize,
}
