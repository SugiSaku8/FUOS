use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use s_field::OceanField;

use crate::grid::{GridData, GridReader};
use crate::reader::BoundaryReader;

/// SOFパッケージのマニフェスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SofManifest {
    pub name: String,
    pub version: u32,
}

/// SOFパッケージReader
pub struct SofPackageReader {
    root: PathBuf,
}

impl SofPackageReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            root: path.as_ref().to_path_buf(),
        })
    }

    /// manifest.json を読む
    pub fn read_manifest(&self) -> Result<SofManifest> {
        let path = self.root.join("manifest.json");

        let text = fs::read_to_string(path)?;

        Ok(serde_json::from_str(&text)?)
    }

    /// grid.snc
    pub fn read_grid(&self) -> Result<GridData> {
        let mut reader = GridReader::open(self.root.join("grid.snc"))?;
        reader.read()
    }

    /// boundary.snc
    pub fn read_boundary(&self) -> Result<OceanField> {
        let mut reader = BoundaryReader::open(self.root.join("boundary.snc").to_str().unwrap())?;

        reader.read()
    }

    /// 現状では field = boundary
    pub fn read_field(&self) -> Result<OceanField> {
        self.read_boundary()
    }
}
