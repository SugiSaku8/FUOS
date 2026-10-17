use anyhow::Result;

use std::{fs::File, io::Read, path::Path};

pub fn read_f32<P: AsRef<Path>>(path: P) -> Result<Vec<f32>> {
    let mut file = File::open(path)?;

    let mut bytes = Vec::new();

    file.read_to_end(&mut bytes)?;

    let values = bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();

    Ok(values)
}
