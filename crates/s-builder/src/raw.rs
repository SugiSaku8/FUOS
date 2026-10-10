use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub struct RawFieldReader;

impl RawFieldReader {
    pub fn read_f32<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<f32>> {
        let mut file = File::open(path)?;

        let mut bytes = Vec::new();

        file.read_to_end(&mut bytes)?;

        let mut data = Vec::with_capacity(bytes.len() / 4);

        for chunk in bytes.chunks_exact(4) {
            data.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        Ok(data)
    }
}
