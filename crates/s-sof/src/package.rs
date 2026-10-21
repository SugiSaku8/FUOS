//! SOF package management.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    boundary::{BoundaryReader, BoundaryWriter},
    error::Result,
    grid::{GridData, GridReader, GridWriter},
    initial::InitialWriter,
    restart::RestartWriter,
};

use s_field::{Grid4D, OceanField};

/// SOF package manifest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SofManifest {
    /// Format name.
    pub format: String,

    /// Format version.
    pub version: String,

    /// Dataset information.
    pub dataset: DatasetManifest,

    /// Files contained in package.
    pub files: PackageFiles,
}

/// Dataset metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatasetManifest {
    /// Dataset name.
    pub name: String,

    /// Dataset type.
    pub dataset_type: String,
}

/// Files inside SOF package.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageFiles {
    /// Grid dataset.
    pub grid: Option<String>,

    /// Boundary dataset.
    pub boundary: Option<String>,

    /// Initial condition dataset.
    pub initial: Option<String>,

    /// Restart dataset.
    pub restart: Option<String>,

    /// External forcing dataset.
    pub forcing: Option<String>,

    /// Optional high-resolution terrain dataset, separate from the solver grid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<String>,
}

/// A SOF package directory.
pub struct SofPackage {
    root: PathBuf,

    manifest: SofManifest,
}

impl SofPackage {
    /// Creates a new SOF package.
    pub fn create<P: AsRef<Path>>(path: P, manifest: SofManifest) -> Result<Self> {
        let root = path.as_ref().to_path_buf();

        fs::create_dir_all(&root)?;

        let text = serde_json::to_string_pretty(&manifest)?;

        fs::write(root.join("manifest.json"), text)?;

        Ok(Self { root, manifest })
    }

    /// Opens an existing SOF package.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path.as_ref().to_path_buf();

        let text = fs::read_to_string(root.join("manifest.json"))?;

        let manifest = serde_json::from_str(&text)?;

        Ok(Self { root, manifest })
    }

    /// Returns manifest.
    pub fn manifest(&self) -> &SofManifest {
        &self.manifest
    }

    /// Returns package path.
    pub fn path(&self) -> &Path {
        &self.root
    }

    // --------------------------------------------------
    // Existence
    // --------------------------------------------------

    /// Checks grid existence.
    pub fn has_grid(&self) -> bool {
        self.manifest.files.grid.is_some()
    }

    /// Checks boundary existence.
    pub fn has_boundary(&self) -> bool {
        self.manifest.files.boundary.is_some()
    }

    /// Checks high-resolution terrain existence.
    pub fn has_terrain(&self) -> bool {
        self.manifest.files.terrain.is_some()
    }

    // --------------------------------------------------
    // Path
    // --------------------------------------------------

    /// Grid file path.
    pub fn grid_path(&self) -> Option<PathBuf> {
        self.manifest.files.grid.as_ref().map(|v| self.root.join(v))
    }

    /// Boundary file path.
    pub fn boundary_path(&self) -> Option<PathBuf> {
        self.manifest
            .files
            .boundary
            .as_ref()
            .map(|v| self.root.join(v))
    }

    /// High-resolution terrain file path.
    pub fn terrain_path(&self) -> Option<PathBuf> {
        self.manifest.files.terrain.as_ref().map(|v| self.root.join(v))
    }

    // --------------------------------------------------
    // Grid
    // --------------------------------------------------

    /// Writes grid dataset.
    pub fn write_grid(
        &self,
        grid: &Grid4D,
        latitude: &[f32],
        longitude: &[f32],
        depth: &[f32],
        bathymetry: &[f32],
    ) -> Result<()> {
        let filename = self
            .manifest
            .files
            .grid
            .clone()
            .unwrap_or("grid.snc".into());

        let path = self.root.join(filename);

        let mut writer = GridWriter::create(path)?;

        writer.write(grid, latitude, longitude, depth, bathymetry)?;

        writer.finish()?;

        Ok(())
    }

    /// Reads grid dataset.
    pub fn read_grid(&self) -> Result<GridData> {
        let path = self.grid_path().expect("grid dataset is not defined");

        let mut reader = GridReader::open(path)?;

        reader.read()
    }

    /// Reads optional high-resolution terrain dataset.
    pub fn read_terrain(&self) -> Result<GridData> {
        let path = self.terrain_path().expect("terrain dataset is not defined");
        let mut reader = GridReader::open(path)?;
        reader.read()
    }

    // --------------------------------------------------
    // Boundary
    // --------------------------------------------------

    /// Writes boundary dataset.
    pub fn write_boundary(&self, field: &OceanField) -> Result<()> {
        let filename = self
            .manifest
            .files
            .boundary
            .clone()
            .unwrap_or("boundary.snc".into());

        let path = self.root.join(filename);

        let mut writer = BoundaryWriter::create(path)?;

        writer.write(field)?;

        writer.finish()?;

        Ok(())
    }

    /// Reads boundary dataset.
    pub fn read_boundary(&self) -> Result<OceanField> {
        let path = self
            .boundary_path()
            .expect("boundary dataset is not defined");

        let mut reader = BoundaryReader::open(path.to_str().unwrap())?;

        reader.read()
    }
    pub fn write_initial(&self, field: &OceanField) -> Result<()> {
        let path = self.root.join("initial.snc");

        InitialWriter::write(path, field)
    }

    pub fn write_restart(&self, field: &OceanField) -> Result<()> {
        let path = self.root.join("restart.snc");

        RestartWriter::write(path, field)
    }
}
