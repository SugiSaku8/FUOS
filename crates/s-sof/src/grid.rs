use std::path::Path;

use s_field::Grid4D;

use s_io::{Metadata, SioReader, SioWriter};

use crate::error::Result;

// ==================================================
// Grid Writer
// ==================================================

pub struct GridWriter {
    writer: SioWriter,
}

impl GridWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            writer: SioWriter::create(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn write(
        &mut self,
        grid: &Grid4D,
        latitude: &[f32],
        longitude: &[f32],
        depth: &[f32],
        bathymetry: &[f32],
    ) -> Result<()> {
        let metadata = Metadata {
            name: "SOF Grid Dataset".into(),
            version: "0.1.0".into(),
            description: "Grid information for ocean simulation".into(),
        };

        self.writer.write_metadata(&metadata)?;

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

        self.writer.write_array("latitude", latitude)?;

        self.writer.write_array("longitude", longitude)?;

        self.writer.write_array("depth", depth)?;

        self.writer.write_array("bathymetry", bathymetry)?;

        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        self.writer.finish()?;

        Ok(())
    }
}

// ==================================================
// Grid Data
// ==================================================

#[derive(Debug)]
pub struct GridData {
    pub size: [usize; 4],

    pub spacing: [f32; 4],

    pub latitude: Vec<f32>,

    pub longitude: Vec<f32>,

    pub depth: Vec<f32>,

    pub bathymetry: Vec<f32>,
}

// ==================================================
// Grid Reader
// ==================================================

pub struct GridReader {
    reader: SioReader,
}

impl GridReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            reader: SioReader::open(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn read(&mut self) -> Result<GridData> {
        Ok(GridData {
            size: self.read_size()?,

            spacing: self.read_spacing()?,

            latitude: self.read_latitude()?,

            longitude: self.read_longitude()?,

            depth: self.read_depth()?,

            bathymetry: self.read_bathymetry()?,
        })
    }

    fn read_size(&mut self) -> Result<[usize; 4]> {
        let v = self.reader.read_array("grid_size")?;

        Ok([v[0] as usize, v[1] as usize, v[2] as usize, v[3] as usize])
    }

    fn read_spacing(&mut self) -> Result<[f32; 4]> {
        let v = self.reader.read_array("grid_spacing")?;

        Ok([v[0], v[1], v[2], v[3]])
    }

    fn read_latitude(&mut self) -> Result<Vec<f32>> {
        Ok(self.reader.read_array("latitude")?)
    }

    fn read_longitude(&mut self) -> Result<Vec<f32>> {
        Ok(self.reader.read_array("longitude")?)
    }

    fn read_depth(&mut self) -> Result<Vec<f32>> {
        Ok(self.reader.read_array("depth")?)
    }

    fn read_bathymetry(&mut self) -> Result<Vec<f32>> {
        Ok(self.reader.read_array("bathymetry")?)
    }
}
