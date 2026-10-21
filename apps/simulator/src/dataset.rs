use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use s_field::{
    BoundaryField, Coordinates, Field4D, Grid4D, Metadata, OceanField, SalinityField,
    TemperatureField, VelocityField,
};
use s_sof::{
    builder::SofPackageBuilder,
    grid::{GridData, GridReader},
    package::SofPackage,
    package_reader::SofPackageReader,
};
use std::{fs, path::Path};

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct TerrainData {
    pub longitude: Vec<f32>,
    pub latitude: Vec<f32>,
    pub bathymetry: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

impl TerrainData {
    pub fn from_grid(grid: GridData) -> Result<Self> {
        let width = grid.longitude.len();
        let height = grid.latitude.len();
        ensure!(width >= 2 && height >= 2, "地形格子は2×2以上必要です");
        ensure!(grid.bathymetry.len() == width * height,
            "bathymetry サイズ不一致: {} != {}", grid.bathymetry.len(), width * height);
        ensure!(grid.longitude.iter().chain(grid.latitude.iter()).chain(grid.bathymetry.iter()).all(|v| v.is_finite()),
            "地形データに NaN / Infinity が含まれています");
        Ok(Self {
            longitude: grid.longitude,
            latitude: grid.latitude,
            bathymetry: grid.bathymetry,
            width,
            height,
        })
    }

    pub fn max_depth_m(&self) -> f32 {
        self.bathymetry.iter().map(|v| (-*v).max(0.0)).fold(0.0, f32::max)
    }

    pub fn latitude_step(&self) -> f32 {
        average_step(&self.latitude).max(1.0e-6)
    }

    pub fn longitude_step(&self) -> f32 {
        average_step(&self.longitude).max(1.0e-6)
    }
}

#[derive(Debug)]
pub struct DatasetBundle {
    pub terrain: TerrainData,
    pub field: OceanField,
    pub depths: Vec<f32>,
}

#[derive(Deserialize)]
struct SourceCoordinates {
    depth: Vec<f32>,
    latitude: Vec<f64>,
    longitude: Vec<f64>,
}

pub fn build(config: &Config) -> Result<DatasetBundle> {
    println!("[1/7] 設定・入力データを読み込みます");
    let terrain_path = config.resolve(&config.input.terrain_grid);
    let mut terrain_reader = GridReader::open(&terrain_path)
        .with_context(|| format!("地形SOFを開けません: {}", terrain_path.display()))?;
    let terrain = TerrainData::from_grid(
        terrain_reader.read().context("地形SOFの読み込みに失敗しました")?,
    )?;

    let coordinate_path = config.resolve(&config.input.source_coordinates);
    let coordinate_bytes = fs::read(&coordinate_path)
        .with_context(|| format!("座標JSONを読めません: {}", coordinate_path.display()))?;
    let source_coordinates: SourceCoordinates = serde_json::from_slice(&coordinate_bytes)
        .with_context(|| format!("座標JSONが不正です: {}", coordinate_path.display()))?;
    ensure!(source_coordinates.depth.len() >= 2, "元データには深度が2層以上必要です");
    ensure!(source_coordinates.latitude.len() >= 2 && source_coordinates.longitude.len() >= 2,
        "元データの緯度・経度格子は2点以上必要です");
    ensure!(strictly_increasing(&source_coordinates.depth), "元データの深度は昇順である必要があります");
    ensure!(strictly_increasing_f64(&source_coordinates.latitude), "元データの緯度は昇順である必要があります");
    ensure!(strictly_increasing_f64(&source_coordinates.longitude), "元データの経度は昇順である必要があります");

    let source_width = source_coordinates.longitude.len();
    let source_height = source_coordinates.latitude.len();
    let source_plane = source_width * source_height;
    let source_count = source_plane * source_coordinates.depth.len();
    let source_u = read_f32(config.resolve(&config.input.velocity_u).as_path())?;
    let source_v = read_f32(config.resolve(&config.input.velocity_v).as_path())?;
    let source_t = read_f32(config.resolve(&config.input.temperature).as_path())?;
    let source_s = read_f32(config.resolve(&config.input.salinity).as_path())?;
    for (name, values) in [
        ("u", &source_u), ("v", &source_v),
        ("temperature", &source_t), ("salinity", &source_s),
    ] {
        ensure!(values.len() == source_count,
            "{} データの要素数が不正です: {} != {} (depth={} × lat={} × lon={})",
            name, values.len(), source_count, source_coordinates.depth.len(), source_height, source_width);
    }

    let interval = config.grid.depth_interval_m;
    let max_depth = terrain.max_depth_m();
    let max_sim_depth = (max_depth / interval).ceil() * interval;
    let depth_count = (max_sim_depth / interval).round() as usize + 1;
    let depths = (0..depth_count).map(|i| i as f32 * interval).collect::<Vec<_>>();

    // The Metal solver uses metres for all three spatial grid spacings.
    let mean_lat = (source_coordinates.latitude[0]
        + source_coordinates.latitude[source_height - 1]) * 0.5;
    let dx_m = average_step_f64(&source_coordinates.longitude) as f32
        * 111_320.0 * (mean_lat.to_radians().cos() as f32).abs().max(0.01);
    let dy_m = average_step_f64(&source_coordinates.latitude) as f32 * 110_540.0;
    let grid = Grid4D::new([1, depth_count, source_height, source_width], [1.0, interval, dy_m, dx_m]);
    let coordinates = Coordinates::new(
        grid,
        source_coordinates.latitude.clone(),
        source_coordinates.longitude.clone(),
        depths.clone(),
        vec![0.0],
    );

    println!("[2/7] 地形に基づく格子: {} × {} × {} (lon × lat × depth)", source_width, source_height, depth_count);
    println!("      深度 0..{:.0}m / Δz={:.0}m / dx={:.1}m / dy={:.1}m", max_sim_depth, interval, dx_m, dy_m);

    let mut u = expand_profile(&source_u, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, true);
    let mut v = expand_profile(&source_v, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, true);
    let mut t = expand_profile(&source_t, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, false);
    let mut s = expand_profile(&source_s, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, false);
    let mut w = vec![0.0_f32; grid.len()];

    // The source and terrain grids have different resolutions; map to the nearest terrain cell.
    let masked_cells = apply_bathymetry_mask(
        &mut u, &mut v, &mut w, &mut t, &mut s, &source_coordinates, &depths, &terrain,
    );
    let mut velocity = VelocityField::with_coordinates(grid.clone(), coordinates.clone());
    velocity.u_mut().replace(u);
    velocity.v_mut().replace(v);
    velocity.w_mut().replace(w);
    let temperature = TemperatureField::from_field(Field4D::from_vec(grid.clone(), t));
    let salinity = SalinityField::from_field(Field4D::from_vec(grid.clone(), s));

    let mut metadata = Metadata::new();
    metadata.set_dataset_name(config.name.clone());
    metadata.set_title("Suruga Bay ocean simulation initial state");
    metadata.set_institution("Suruga Ocean Simulator");
    metadata.set_source("Configured raw velocity, temperature, salinity and bathymetry inputs");
    metadata.set_history("Generated by apps/simulator dataset preparation");
    metadata.set_comment("Velocity below the deepest source layer is initialized using exponential attenuation; it is not source-observed deep data.");
    let field = OceanField::new(
        coordinates, metadata, velocity, temperature, salinity, BoundaryField::zeros(grid),
    );

    let output_root = config.output_root();
    fs::create_dir_all(&output_root)
        .with_context(|| format!("出力先を作成できません: {}", output_root.display()))?;
    let input_package_path = output_root.join("input.sof");
    write_package(&input_package_path, &config.name, &field, &terrain, &depths, interval)?;
    println!("[3/7] 初期状態をSOF化: {}", input_package_path.display());

    // Use the serialized SOF as the actual simulation input.
    let package = SofPackageReader::open(&input_package_path)?;
    let field_from_sof = package.read_initial()
        .context("生成したSOFのinitial.sncを再読込できません")?;
    let terrain_package = SofPackage::open(&input_package_path)
        .context("生成したSOFパッケージを開けません")?;
    let terrain_from_sof = TerrainData::from_grid(
        terrain_package.read_terrain().context("生成したSOFのterrain.sncを再読込できません")?,
    )?;
    println!("      source grid={}×{}×{}, masked land/sub-bottom cells={}",
        source_width, source_height, depth_count, masked_cells);

    Ok(DatasetBundle { terrain: terrain_from_sof, field: field_from_sof, depths })
}

pub fn write_package(
    output: &Path,
    name: &str,
    field: &OceanField,
    terrain: &TerrainData,
    depths: &[f32],
    depth_interval: f32,
) -> Result<()> {
    write_result_package(output, name, field, field, terrain, depths, depth_interval)
}

/// Write a package that preserves the original initial state and stores the current
/// simulation state in restart.snc (and boundary.snc).
pub fn write_result_package(
    output: &Path,
    name: &str,
    initial_field: &OceanField,
    current_field: &OceanField,
    terrain: &TerrainData,
    depths: &[f32],
    depth_interval: f32,
) -> Result<()> {
    // SOF grid.snc must share the solver grid axes with initial.snc/restart.snc.
    // The high-resolution source terrain remains available as the configured render input;
    // a nearest-neighbour bathymetry sample is stored alongside the solver grid in SOF.
    let coordinates = current_field.coordinates();
    let solver_grid = *current_field.velocity().grid();
    let latitude = coordinates.latitude().iter().map(|v| *v as f32).collect::<Vec<_>>();
    let longitude = coordinates.longitude().iter().map(|v| *v as f32).collect::<Vec<_>>();
    ensure!((solver_grid.dz() - depth_interval).abs() < 1.0e-3,
        "SOFの鉛直格子間隔が設定値と一致しません: {} != {}", solver_grid.dz(), depth_interval);
    let bathymetry = resample_bathymetry(terrain, &latitude, &longitude);
    let plane = latitude.len() * longitude.len();
    let zeros = vec![0.0_f32; plane];
    let terrain_grid = Grid4D::new(
        [1, 1, terrain.height, terrain.width],
        [1.0, 1.0, terrain.latitude_step() * 110_540.0,
            terrain.longitude_step() * 111_320.0 * (terrain.latitude.iter().map(|v| *v as f64).sum::<f64>() / terrain.latitude.len() as f64).to_radians().cos().abs() as f32],
    );
    SofPackageBuilder::new(name)
        .grid(solver_grid, latitude, longitude, depths.to_vec(), bathymetry)
        .terrain(terrain_grid, terrain.latitude.clone(), terrain.longitude.clone(), vec![0.0], terrain.bathymetry.clone())
        .boundary(current_field.clone())
        .initial(initial_field.clone())
        .restart(current_field.clone())
        .forcing(zeros.clone(), zeros.clone(), zeros)
        .build(output)
        .with_context(|| format!("SOFパッケージを書き込めません: {}", output.display()))?;
    Ok(())
}

fn resample_bathymetry(terrain: &TerrainData, latitude: &[f32], longitude: &[f32]) -> Vec<f32> {
    let mut values = Vec::with_capacity(latitude.len() * longitude.len());
    for &lat in latitude {
        let ty = nearest_index(&terrain.latitude, lat).min(terrain.height - 1);
        for &lon in longitude {
            let tx = nearest_index(&terrain.longitude, lon).min(terrain.width - 1);
            values.push(terrain.bathymetry[ty * terrain.width + tx]);
        }
    }
    values
}

fn read_f32(path: &Path) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("バイナリを読めません: {}", path.display()))?;
    ensure!(bytes.len() % 4 == 0, "f32バイナリのサイズが4の倍数ではありません: {}", path.display());
    let values = bytes.chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    ensure!(values.iter().all(|v| v.is_finite()), "非有限値を含むバイナリです: {}", path.display());
    Ok(values)
}

fn expand_profile(
    source: &[f32], source_depths: &[f32], target_depths: &[f32], plane: usize,
    decay_scale: f32, attenuate_deep: bool,
) -> Vec<f32> {
    let mut out = Vec::with_capacity(target_depths.len() * plane);
    let last_source_depth = *source_depths.last().unwrap_or(&0.0);
    for &depth in target_depths {
        if depth > last_source_depth {
            let last_offset = (source_depths.len() - 1) * plane;
            let factor = if attenuate_deep {
                (-(depth - last_source_depth) / decay_scale).exp()
            } else { 1.0 };
            out.extend(source[last_offset..last_offset + plane].iter().map(|v| *v * factor));
            continue;
        }
        let upper = source_depths.iter().position(|d| *d >= depth).unwrap_or(source_depths.len() - 1);
        let lower = upper.saturating_sub(1);
        let span = source_depths[upper] - source_depths[lower];
        let weight = if span > 0.0 { (depth - source_depths[lower]) / span } else { 0.0 };
        let low_offset = lower * plane;
        let high_offset = upper * plane;
        for i in 0..plane {
            out.push(source[low_offset + i] * (1.0 - weight) + source[high_offset + i] * weight);
        }
    }
    out
}

fn apply_bathymetry_mask(
    u: &mut [f32], v: &mut [f32], w: &mut [f32], t: &mut [f32], s: &mut [f32],
    source: &SourceCoordinates, depths: &[f32], terrain: &TerrainData,
) -> usize {
    let width = source.longitude.len();
    let height = source.latitude.len();
    let plane = width * height;
    let mut masked = 0usize;
    for (z, &depth) in depths.iter().enumerate() {
        for y in 0..height {
            for x in 0..width {
                let tx = nearest_index(&terrain.longitude, source.longitude[x] as f32).min(terrain.width - 1);
                let ty = nearest_index(&terrain.latitude, source.latitude[y] as f32).min(terrain.height - 1);
                let elevation = terrain.bathymetry[ty * terrain.width + tx];
                if elevation >= 0.0 || depth > -elevation {
                    let i = z * plane + y * width + x;
                    u[i] = 0.0;
                    v[i] = 0.0;
                    w[i] = 0.0;
                    t[i] = 0.0;
                    s[i] = 0.0;
                    masked += 1;
                }
            }
        }
    }
    masked
}

fn nearest_index(values: &[f32], value: f32) -> usize {
    values.iter().enumerate()
        .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
        .map(|(i, _)| i).unwrap_or(0)
}
fn average_step(values: &[f32]) -> f32 {
    if values.len() < 2 { return 1.0; }
    values.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / (values.len() - 1) as f32
}
fn average_step_f64(values: &[f64]) -> f64 {
    if values.len() < 2 { return 1.0; }
    values.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>() / (values.len() - 1) as f64
}
fn strictly_increasing(values: &[f32]) -> bool { values.windows(2).all(|w| w[1] > w[0]) }
fn strictly_increasing_f64(values: &[f64]) -> bool { values.windows(2).all(|w| w[1] > w[0]) }
