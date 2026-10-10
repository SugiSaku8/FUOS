use std::path::{Path, PathBuf};

use s_field::OceanField;

use crate::{
    error::Result,
    forcing::{ForcingData, ForcingReader},
    grid::{GridData, GridReader},
    initial::InitialReader,
    reader::BoundaryReader,
    restart::RestartReader,
};

/// Reader for SOF package.
pub struct SofPackageReader {
    root: PathBuf,
}

impl SofPackageReader {
    /// Opens SOF package.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            root: path.as_ref().to_path_buf(),
        })
    }

    /// Reads grid dataset.
    pub fn read_grid(&self) -> Result<GridData> {
        let mut reader = GridReader::open(self.root.join("grid.snc"))?;

        reader.read()
    }

    /// Reads boundary dataset.
    pub fn read_boundary(&self) -> Result<OceanField> {
        let mut reader = BoundaryReader::open(self.root.join("boundary.snc").to_str().unwrap())?;

        reader.read()
    }

    /// Reads initial condition.
    pub fn read_initial(&self) -> Result<OceanField> {
        InitialReader::read(self.root.join("initial.snc"))
    }

    /// Reads restart state.
    pub fn read_restart(&self) -> Result<OceanField> {
        RestartReader::read(self.root.join("restart.snc"))
    }

    /// Reads surface forcing.
    pub fn read_forcing(&self) -> Result<ForcingData> {
        let mut reader = ForcingReader::open(self.root.join("forcing.snc"))?;

        reader.read()
    }

    /// Reads main ocean field.
    ///
    /// Currently boundary is used as main field.
    pub fn read_field(&self) -> Result<OceanField> {
        self.read_initial()
    }
}
