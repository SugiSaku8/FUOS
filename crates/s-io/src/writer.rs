use std::fs::File;

use std::io::{Seek, SeekFrom, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{error::SioError, format::*, types::*};

pub struct SioWriter {
    file: File,

    datasets: Vec<DatasetInfo>,
}

impl SioWriter {
    pub fn create(path: &str) -> Result<Self, SioError> {
        let mut file = File::create(path)?;

        file.write_all(MAGIC)?;

        file.write_u32::<LittleEndian>(1)?;

        file.seek(SeekFrom::Start(HEADER_SIZE as u64))?;

        Ok(Self {
            file,

            datasets: Vec::new(),
        })
    }

    pub fn write_metadata(&mut self, metadata: &Metadata) -> Result<(), SioError> {
        let data = serde_json::to_vec(metadata)?;

        self.file.write_u64::<LittleEndian>(data.len() as u64)?;

        self.file.write_all(&data)?;

        Ok(())
    }

    pub fn write_array(&mut self, name: &str, values: &[f32]) -> Result<(), SioError> {
        let offset = self.file.stream_position()?;

        for value in values {
            self.file.write_f32::<LittleEndian>(*value)?;
        }

        let info = DatasetInfo {
            name: name.to_string(),

            offset,

            size: (values.len() * std::mem::size_of::<f32>()) as u64,

            element_count: values.len(),
        };

        self.datasets.push(info);

        Ok(())
    }

    pub fn finish(mut self) -> Result<(), SioError> {
        let table = serde_json::to_vec(&self.datasets)?;

        // dataset table
        self.file.write_all(&table)?;

        // table size
        self.file.write_u64::<LittleEndian>(table.len() as u64)?;

        // magic at end
        self.file.write_all(DATASET_TABLE_MAGIC)?;

        self.file.flush()?;

        Ok(())
    }
}
