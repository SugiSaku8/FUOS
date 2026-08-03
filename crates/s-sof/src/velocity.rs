use std::path::Path;

use s_field::{Coordinates, Grid4D, VelocityField};

use s_io::{Metadata, SioReader, SioWriter};

use crate::error::Result;

// ==================================================
// Velocity Writer
// ==================================================

pub struct VelocityWriter {
    writer: SioWriter,
}

impl VelocityWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            writer: SioWriter::create(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn write(&mut self, velocity: &VelocityField) -> Result<()> {
        let metadata = Metadata {
            name: "Velocity Field".into(),
            version: "0.1.0".into(),
            description: "Ocean velocity field (u,v,w)".into(),
        };

        self.writer.write_metadata(&metadata)?;

        let size = velocity.grid().size();

        self.writer.write_array(
            "grid_size",
            &[
                size[0] as f32,
                size[1] as f32,
                size[2] as f32,
                size[3] as f32,
            ],
        )?;

        let spacing = velocity.grid().spacing();

        self.writer.write_array("grid_spacing", &spacing)?;

        self.writer.write_array("u", velocity.u().as_slice())?;

        self.writer.write_array("v", velocity.v().as_slice())?;

        self.writer.write_array("w", velocity.w().as_slice())?;

        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        self.writer.finish()?;

        Ok(())
    }
}

// ==================================================
// Velocity Reader
// ==================================================

pub struct VelocityReader {
    reader: SioReader,
}

impl VelocityReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            reader: SioReader::open(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn read(&mut self, coordinates: Coordinates) -> Result<VelocityField> {
        let size = self.read_size()?;

        let spacing = self.read_spacing()?;

        let grid = Grid4D::new(size, spacing);

        let u = self.reader.read_array("u")?;

        let v = self.reader.read_array("v")?;

        let w = self.reader.read_array("w")?;

        let mut velocity = VelocityField::new(grid.clone());

        velocity.u_mut().replace(u);

        velocity.v_mut().replace(v);

        velocity.w_mut().replace(w);

        velocity.set_coordinates(coordinates);

        Ok(velocity)
    }

    fn read_size(&mut self) -> Result<[usize; 4]> {
        let v = self.reader.read_array("grid_size")?;

        Ok([v[0] as usize, v[1] as usize, v[2] as usize, v[3] as usize])
    }

    fn read_spacing(&mut self) -> Result<[f32; 4]> {
        let v = self.reader.read_array("grid_spacing")?;

        Ok([v[0], v[1], v[2], v[3]])
    }
}
