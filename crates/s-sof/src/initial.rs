use std::path::Path;

use s_field::OceanField;

use crate::error::Result;

use crate::reader::BoundaryReader;
use crate::writer::BoundaryWriter;

pub struct InitialWriter;

impl InitialWriter {
    pub fn write<P: AsRef<Path>>(path: P, field: &OceanField) -> Result<()> {
        let mut writer = BoundaryWriter::create(path)?;

        writer.write(field)?;

        writer.finish()
    }
}

pub struct InitialReader;

impl InitialReader {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<OceanField> {
        let mut reader = BoundaryReader::open(path.as_ref().to_str().unwrap())?;

        reader.read()
    }
}
