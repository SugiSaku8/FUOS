use std::path::{Path, PathBuf};

use crate::{error::Result, grid::GridData, package::SofPackage, reader::BoundaryReader};

use s_field::OceanField;

pub struct SofPackageReader {
    path: PathBuf,
}

impl SofPackageReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    pub fn read_grid(&self) -> Result<GridData> {
        let path = self.path.join("grid.snc");

        let mut reader = crate::grid::GridReader::open(path)?;

        reader.read()
    }

    pub fn read_boundary(&self) -> Result<OceanField> {
        let path = self.path.join("boundary.snc");

        let mut reader = BoundaryReader::open(path.to_str().unwrap())?;

        reader.read()
    }
}
