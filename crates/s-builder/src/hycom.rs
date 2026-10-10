use std::fs;

use anyhow::Result;

use s_field::{Grid4D, ScalarField};

pub struct HycomDataset {
    pub grid: Grid4D,

    pub temperature: ScalarField<f32>,

    pub salinity: ScalarField<f32>,

    pub u: ScalarField<f32>,

    pub v: ScalarField<f32>,
}

pub struct HycomLoader;

impl HycomLoader {
    pub fn load(path: &str) -> Result<HycomDataset> {
        let meta = fs::read_to_string(format!("{}/metadata.json", path))?;

        let json: serde_json::Value = serde_json::from_str(&meta)?;

        let width = json["width"].as_u64().unwrap() as usize;

        let height = json["height"].as_u64().unwrap() as usize;

        let depth = json["depth"].as_u64().unwrap() as usize;

        let time = json["time"].as_u64().unwrap() as usize;

        let grid = Grid4D::new([width, height, depth, time], [1.0, 1.0, 1.0, 1.0]);

        Ok(HycomDataset {
            temperature: Self::read_field(path, "temperature.bin", grid.clone())?,

            salinity: Self::read_field(path, "salinity.bin", grid.clone())?,

            u: Self::read_field(path, "u.bin", grid.clone())?,

            v: Self::read_field(path, "v.bin", grid.clone())?,

            grid,
        })
    }

    fn read_field(path: &str, file: &str, grid: Grid4D) -> Result<ScalarField<f32>> {
        let bytes = fs::read(format!("{}/{}", path, file))?;

        let values: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|x| f32::from_le_bytes([x[0], x[1], x[2], x[3]]))
            .collect();

        let mut field = ScalarField::zeros(grid);

        field.replace(values);

        Ok(field)
    }
}
