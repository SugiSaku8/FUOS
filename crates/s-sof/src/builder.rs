use std::fs;
use std::path::{Path, PathBuf};

use s_field::OceanField;

use crate::{
    boundary::BoundaryWriter,
    error::Result,
    package::{PackageFiles, SofManifest, SofPackage},
};
/// Builder for creating SOF packages.
pub struct SofPackageBuilder {
    name: String,

    boundary: Option<OceanField>,

    grid: Option<PathBuf>,

    forcing: Option<PathBuf>,

    restart: Option<PathBuf>,

    initial: Option<PathBuf>,
}

impl SofPackageBuilder {
    /// Creates a new builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),

            boundary: None,

            grid: None,

            forcing: None,

            restart: None,

            initial: None,
        }
    }

    /// Adds boundary field.
    pub fn boundary(mut self, field: OceanField) -> Self {
        self.boundary = Some(field);

        self
    }

    /// Adds grid file.
    pub fn grid<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.grid = Some(path.as_ref().into());

        self
    }

    /// Adds forcing file.
    pub fn forcing<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.forcing = Some(path.as_ref().into());

        self
    }

    /// Adds restart file.
    pub fn restart<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.restart = Some(path.as_ref().into());

        self
    }

    /// Adds initial condition file.
    pub fn initial<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.initial = Some(path.as_ref().into());

        self
    }

    /// Builds SOF package.
    pub fn build<P: AsRef<Path>>(self, output: P) -> Result<SofPackage> {
        let output = output.as_ref();

        fs::create_dir_all(output)?;

        //
        // boundary.snc
        //

        if let Some(field) = self.boundary {
            let path = output.join("boundary.snc");

            let mut writer = BoundaryWriter::create(&path)?;

            writer.write(&field)?;

            writer.finish()?;
        }

        //
        // Manifest
        //

        let files = PackageFiles {
            grid: self.grid.map(|p| p.to_string_lossy().to_string()),

            boundary: Some("boundary.snc".to_string()),

            forcing: self.forcing.map(|p| p.to_string_lossy().to_string()),

            restart: self.restart.map(|p| p.to_string_lossy().to_string()),

            initial: self.initial.map(|p| p.to_string_lossy().to_string()),
        };

        let manifest = SofManifest {
            format: "SOF".to_string(),

            version: "0.1.0".to_string(),

            dataset: crate::package::DatasetManifest {
                name: self.name,

                dataset_type: "Boundary".to_string(),
            },

            files,
        };

        SofPackage::create(output, manifest)
    }
}
