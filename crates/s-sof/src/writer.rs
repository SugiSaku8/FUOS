use std::path::Path;

use s_field::{Field, OceanField};

use s_io::{Metadata, SioWriter};

use crate::{dataset::SofDataset, error::Result};

/// Writer for SOF boundary datasets.
pub struct BoundaryWriter {
    writer: SioWriter,
}

impl BoundaryWriter {
    /// Creates a new boundary writer.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let writer = SioWriter::create(path.as_ref().to_str().unwrap())?;

        Ok(Self { writer })
    }

    /// Writes an OceanField into SOF format.
    pub fn write(&mut self, field: &OceanField) -> Result<()> {
        //
        // Metadata
        //

        let metadata = Metadata {
            name: "SOF Boundary Dataset".to_string(),

            version: "0.1.0".to_string(),

            description: "Suruga Ocean Format boundary data".to_string(),
        };

        self.writer.write_metadata(&metadata)?;

        //
        // SOF Dataset description
        //

        let dataset = SofDataset::boundary();

        let dataset_json = serde_json::to_string(&dataset).unwrap();

        let dataset_bytes: Vec<f32> = dataset_json.as_bytes().iter().map(|v| *v as f32).collect();

        self.writer.write_array("sof_dataset", &dataset_bytes)?;

        //
        // Grid information
        //

        let grid = field.velocity().grid();

        let size = grid.size();

        self.writer.write_array(
            "grid_size",
            &[
                size[0] as f32,
                size[1] as f32,
                size[2] as f32,
                size[3] as f32,
            ],
        )?;

        let spacing = grid.spacing();

        self.writer.write_array(
            "grid_spacing",
            &[spacing[0], spacing[1], spacing[2], spacing[3]],
        )?;

        //
        // Coordinates
        //

        let coordinates = field.coordinates();

        let latitude: Vec<f32> = coordinates.latitude().iter().map(|v| *v as f32).collect();

        let longitude: Vec<f32> = coordinates.longitude().iter().map(|v| *v as f32).collect();

        let time: Vec<f32> = coordinates.time().iter().map(|v| *v as f32).collect();

        self.writer.write_array("latitude", &latitude)?;

        self.writer.write_array("longitude", &longitude)?;

        self.writer.write_array("depth", coordinates.depth())?;

        self.writer.write_array("time", &time)?;

        //
        // Velocity
        //

        self.writer
            .write_array("velocity_u", field.velocity().u().as_slice())?;

        self.writer
            .write_array("velocity_v", field.velocity().v().as_slice())?;

        self.writer
            .write_array("velocity_w", field.velocity().w().as_slice())?;

        //
        // Temperature
        //

        self.writer
            .write_array("temperature", field.temperature().field().as_slice())?;

        //
        // Salinity
        //

        self.writer
            .write_array("salinity", field.salinity().field().as_slice())?;

        Ok(())
    }

    /// Finalizes writing.
    pub fn finish(self) -> Result<()> {
        self.writer.finish()?;

        Ok(())
    }
}
