use std::fs;
use std::path::Path;

use s_field::{Grid4D, OceanField};

use crate::{
    boundary::BoundaryWriter,
    error::Result,
    forcing::ForcingWriter,
    grid::GridWriter,
    initial::InitialWriter,
    package::{DatasetManifest, PackageFiles, SofManifest, SofPackage},
    restart::RestartWriter,
};

/// Builder for creating complete SOF packages.
pub struct SofPackageBuilder {
    name: String,

    grid: Option<GridDataBuilder>,

    boundary: Option<OceanField>,

    initial: Option<OceanField>,

    restart: Option<OceanField>,

    forcing: Option<ForcingBuilder>,
}

struct GridDataBuilder {
    grid: Grid4D,

    latitude: Vec<f32>,

    longitude: Vec<f32>,

    depth: Vec<f32>,

    bathymetry: Vec<f32>,
}

struct ForcingBuilder {
    wind_u: Vec<f32>,

    wind_v: Vec<f32>,

    surface_heat: Vec<f32>,
}

impl SofPackageBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),

            grid: None,

            boundary: None,

            initial: None,

            restart: None,

            forcing: None,
        }
    }

    pub fn grid(
        mut self,
        grid: Grid4D,
        latitude: Vec<f32>,
        longitude: Vec<f32>,
        depth: Vec<f32>,
        bathymetry: Vec<f32>,
    ) -> Self {
        self.grid = Some(GridDataBuilder {
            grid,
            latitude,
            longitude,
            depth,
            bathymetry,
        });

        self
    }

    pub fn boundary(mut self, field: OceanField) -> Self {
        self.boundary = Some(field);

        self
    }

    pub fn initial(mut self, field: OceanField) -> Self {
        self.initial = Some(field);

        self
    }

    pub fn restart(mut self, field: OceanField) -> Self {
        self.restart = Some(field);

        self
    }

    pub fn forcing(mut self, wind_u: Vec<f32>, wind_v: Vec<f32>, surface_heat: Vec<f32>) -> Self {
        self.forcing = Some(ForcingBuilder {
            wind_u,
            wind_v,
            surface_heat,
        });

        self
    }

    pub fn build<P: AsRef<Path>>(self, output: P) -> Result<SofPackage> {
        let output = output.as_ref();

        fs::create_dir_all(output)?;

        // grid.snc

        if let Some(grid) = self.grid {
            let path = output.join("grid.snc");

            let mut writer = GridWriter::create(&path)?;

            writer.write(
                &grid.grid,
                &grid.latitude,
                &grid.longitude,
                &grid.depth,
                &grid.bathymetry,
            )?;

            writer.finish()?;
        }

        // boundary.snc

        if let Some(field) = self.boundary {
            let path = output.join("boundary.snc");

            let mut writer = BoundaryWriter::create(&path)?;

            writer.write(&field)?;

            writer.finish()?;
        }

        // initial.snc

        if let Some(field) = self.initial {
            InitialWriter::write(output.join("initial.snc"), &field)?;
        }

        // restart.snc

        if let Some(field) = self.restart {
            RestartWriter::write(output.join("restart.snc"), &field)?;
        }

        // forcing.snc

        if let Some(forcing) = self.forcing {
            let path = output.join("forcing.snc");

            let mut writer = ForcingWriter::create(&path)?;

            writer.write(&forcing.wind_u, &forcing.wind_v, &forcing.surface_heat)?;

            writer.finish()?;
        }

        let files = PackageFiles {
            grid: Some("grid.snc".into()),

            boundary: Some("boundary.snc".into()),

            initial: Some("initial.snc".into()),

            restart: Some("restart.snc".into()),

            forcing: Some("forcing.snc".into()),
        };

        let manifest = SofManifest {
            format: "SOF".into(),

            version: "0.1.0".into(),

            dataset: DatasetManifest {
                name: self.name,

                dataset_type: "OceanDataset".into(),
            },

            files,
        };

        SofPackage::create(output, manifest)
    }
}
