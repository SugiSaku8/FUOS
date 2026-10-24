//! Generates the solver Grid directly from source coordinates and terrain depth.
use anyhow::{ensure, Result};
use s_field::{Coordinates, Grid4D};
use crate::{dataset::TerrainData, source_bundle::SourceCoordinates};

pub struct GridBuilder;

pub struct GeneratedGrid {
    pub grid: Grid4D,
    pub coordinates: Coordinates,
    pub depths: Vec<f32>,
    pub max_sim_depth_m: f32,
    pub dx_m: f32,
    pub dy_m: f32,
}

impl GridBuilder {
    pub fn build(terrain: &TerrainData, source: &SourceCoordinates, depth_interval_m: f32) -> Result<GeneratedGrid> {
        ensure!(depth_interval_m.is_finite() && depth_interval_m > 0.0, "深度間隔は正の有限値にしてください");
        ensure!(source.depth.len() >= 2 && source.latitude.len() >= 2 && source.longitude.len() >= 2,
            "Grid生成には深度・緯度・経度がそれぞれ2点以上必要です");
        ensure!(source.depth.windows(2).all(|w| w[1] > w[0]), "元データの深度は昇順である必要があります");
        ensure!(source.latitude.windows(2).all(|w| w[1] > w[0]), "元データの緯度は昇順である必要があります");
        ensure!(source.longitude.windows(2).all(|w| w[1] > w[0]), "元データの経度は昇順である必要があります");

        let max_sim_depth_m = (terrain.max_depth_m() / depth_interval_m).ceil() * depth_interval_m;
        let depth_count = (max_sim_depth_m / depth_interval_m).round() as usize + 1;
        let depths = (0..depth_count).map(|i| i as f32 * depth_interval_m).collect::<Vec<_>>();
        let height = source.latitude.len();
        let mean_lat = (source.latitude[0] + source.latitude[height - 1]) * 0.5;
        let dx_m = average_step_f64(&source.longitude) as f32 * 111_320.0
            * (mean_lat.to_radians().cos() as f32).abs().max(0.01);
        let dy_m = average_step_f64(&source.latitude) as f32 * 110_540.0;
        let grid = Grid4D::new(
            [1, depth_count, source.latitude.len(), source.longitude.len()],
            [1.0, depth_interval_m, dy_m, dx_m],
        );
        let coordinates = Coordinates::new(
            grid.clone(), source.latitude.clone(), source.longitude.clone(), depths.clone(), vec![0.0],
        );
        Ok(GeneratedGrid { grid, coordinates, depths, max_sim_depth_m, dx_m, dy_m })
    }
}

fn average_step_f64(values: &[f64]) -> f64 {
    if values.len() < 2 { return 1.0; }
    values.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (values.len() - 1) as f64
}
