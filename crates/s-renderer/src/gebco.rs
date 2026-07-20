use anyhow::{Result, anyhow};

use crate::heightmap::HeightMap;

pub struct GebcoLoader;

impl GebcoLoader {
    pub fn load(path: &str) -> Result<HeightMap> {
        let file = netcdf::open(path)?;

        //
        // elevation取得
        //
        let elevation = file
            .variable("elevation")
            .ok_or_else(|| anyhow!("elevation variable not found"))?;

        let dims = elevation.dimensions();

        if dims.len() != 2 {
            return Err(anyhow!("elevation must be 2D"));
        }

        let height = dims[0].len();

        let width = dims[1].len();

        println!("GEBCO size: {} x {}", width, height);

        //
        // short -> f32
        //
        let values: Vec<i16> = elevation.get_values(..)?;

        let data: Vec<f32> = values
            .into_iter()
            .map(|v| {
                //
                // NoData
                //
                if v == -32767 { 0.0 } else { v as f32 }
            })
            .collect();

        let heightmap = HeightMap::new(width, height, data);

        let (min, max) = heightmap.min_max();

        println!("Elevation range: {} m ～ {} m", min, max);

        Ok(heightmap)
    }
}
