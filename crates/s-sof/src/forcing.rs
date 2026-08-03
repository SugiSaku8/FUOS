use std::path::Path;

use s_io::{Metadata, SioReader, SioWriter};

use crate::error::Result;

pub struct ForcingWriter {
    writer: SioWriter,
}

impl ForcingWriter {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            writer: SioWriter::create(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn write(&mut self, wind_u: &[f32], wind_v: &[f32], surface_heat: &[f32]) -> Result<()> {
        let metadata = Metadata {
            name: "SOF Forcing Dataset".into(),

            version: "0.1.0".into(),

            description: "External forcing data".into(),
        };

        self.writer.write_metadata(&metadata)?;

        self.writer.write_array("wind_u", wind_u)?;

        self.writer.write_array("wind_v", wind_v)?;

        self.writer.write_array("surface_heat", surface_heat)?;

        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        self.writer.finish()?;

        Ok(())
    }
}

pub struct ForcingReader {
    reader: SioReader,
}

impl ForcingReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            reader: SioReader::open(path.as_ref().to_str().unwrap())?,
        })
    }

    pub fn read(&mut self) -> Result<ForcingData> {
        Ok(ForcingData {
            wind_u: self.reader.read_array("wind_u")?,

            wind_v: self.reader.read_array("wind_v")?,

            surface_heat: self.reader.read_array("surface_heat")?,
        })
    }
}

pub struct ForcingData {
    pub wind_u: Vec<f32>,

    pub wind_v: Vec<f32>,

    pub surface_heat: Vec<f32>,
}
