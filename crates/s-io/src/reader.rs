use std::fs::File;

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{error::SioError, format::*, types::*};
pub struct SioReader {
    file: File,

    metadata: Metadata,

    datasets: Vec<DatasetInfo>,
}

impl SioReader {
    pub fn open(path: &str) -> Result<Self, SioError> {
        let mut file = File::open(path)?;

        //
        // Header
        //

        let mut magic = [0u8; 8];

        file.read_exact(&mut magic)?;

        if &magic != MAGIC {
            return Err(SioError::InvalidFormat);
        }

        let _version = file.read_u32::<LittleEndian>()?;

        //
        // Metadata
        //

        file.seek(SeekFrom::Start(HEADER_SIZE as u64))?;

        let metadata_size = file.read_u64::<LittleEndian>()? as usize;

        let mut metadata_buf = vec![0u8; metadata_size];

        file.read_exact(&mut metadata_buf)?;

        let metadata = serde_json::from_slice(&metadata_buf)?;

        //
        // Dataset table
        //

        let file_size = file.metadata()?.len();

        //
        // 最後8byte = DATASET_TABLE_MAGIC
        //

        if file_size < 8 {
            return Err(SioError::InvalidFormat);
        }

        let magic_pos = file_size - 8;

        file.seek(SeekFrom::Start(magic_pos))?;

        let mut table_magic = [0u8; 8];

        file.read_exact(&mut table_magic)?;

        if &table_magic != DATASET_TABLE_MAGIC {
            return Err(SioError::InvalidFormat);
        }

        //
        // その直前8byte = table size
        //

        if magic_pos < 8 {
            return Err(SioError::InvalidFormat);
        }

        let size_pos = magic_pos - 8;

        file.seek(SeekFrom::Start(size_pos))?;

        let table_size = file.read_u64::<LittleEndian>()? as usize;

        //
        // table JSON位置
        //

        if size_pos < table_size as u64 {
            return Err(SioError::InvalidFormat);
        }

        let table_pos = size_pos - table_size as u64;

        file.seek(SeekFrom::Start(table_pos))?;

        let mut table_buf = vec![0u8; table_size];

        file.read_exact(&mut table_buf)?;

        let datasets = serde_json::from_slice(&table_buf)?;

        Ok(Self {
            file,

            metadata,

            datasets,
        })
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn datasets(&self) -> &[DatasetInfo] {
        &self.datasets
    }

    pub fn read_dataset(&mut self, name: &str) -> Result<Vec<f32>, SioError> {
        let dataset = self
            .datasets
            .iter()
            .find(|d| d.name == name)
            .ok_or_else(|| SioError::DatasetNotFound(name.to_string()))?
            .clone();

        self.file.seek(SeekFrom::Start(dataset.offset))?;

        let mut values = Vec::with_capacity(dataset.element_count);

        for _ in 0..dataset.element_count {
            values.push(self.file.read_f32::<LittleEndian>()?);
        }

        Ok(values)
    }
    pub fn read_array(&mut self, name: &str) -> Result<Vec<f32>, SioError> {
        let info = self
            .datasets
            .iter()
            .find(|d| d.name == name)
            .ok_or(SioError::InvalidFormat)?;

        self.file.seek(SeekFrom::Start(info.offset))?;

        let count = info.element_count;

        let mut data = Vec::with_capacity(count);

        for _ in 0..count {
            data.push(self.file.read_f32::<LittleEndian>()?);
        }

        Ok(data)
    }
}
