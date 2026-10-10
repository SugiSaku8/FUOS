use anyhow::Result;
use std::path::Path;

use s_sof::grid::GridReader;

#[derive(Debug)]
pub struct HeightMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

impl HeightMap {
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Self {
        assert_eq!(width * height, data.len());

        Self {
            width,
            height,
            data,
        }
    }

    //
    // SOF loader
    //
    pub fn load_sof<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut reader = GridReader::open(path)?;

        let grid = reader.read()?;

        let width = grid.longitude.len();

        let height = grid.latitude.len();

        println!("SOF heightmap {} x {}", width, height);

        Ok(Self::new(width, height, grid.bathymetry))
    }

    //
    // compatibility
    //
    pub fn get(&self, x: usize, z: usize) -> f32 {
        let index = z * self.width + x;

        self.data[index]
    }

    pub fn min_max(&self) -> (f32, f32) {
        let min = self.data.iter().cloned().fold(f32::INFINITY, f32::min);

        let max = self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        (min, max)
    }

    //
    // temporary
    //
    pub fn test() -> Self {
        let width = 64;
        let height = 64;

        let data = vec![0.0; width * height];

        Self::new(width, height, data)
    }
}
