use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,

    pub version: String,

    pub title: String,

    pub creator: String,

    pub created: String,

    pub grid: Option<String>,

    pub boundary: Option<String>,

    pub restart: Option<String>,
}
