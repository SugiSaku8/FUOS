use anyhow::{Context, Result, ensure};
use s_field::{
    BoundaryField, Field4D, Grid4D, Metadata, OceanField, SalinityField,
    TemperatureField, VelocityField,
};
use s_sof::{
    builder::SofPackageBuilder,
    grid::{GridData, GridReader},
    package::SofPackage,
    package_reader::SofPackageReader,
};
use std::{fs, path::Path};

use crate::source_bundle::{SourceBundle, SourceCoordinates};
use crate::grid_builder::GridBuilder;

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

/// Load an already-generated SOF package without regenerating inputs.
pub fn load_existing_sof(config: &Config) -> Result<DatasetBundle> {
    let input_package_path = config.output_root().join("input.sof");
    ensure!(input_package_path.is_dir(),
        "SOFパッケージがありません: {}。先に cargo run -p simulator -- generate を実行してください。",
        input_package_path.display());

    println!("SOFからシミュレーション入力を読み込みます: {}", input_package_path.display());
    let package = SofPackageReader::open(&input_package_path)
        .context("SOFパッケージを開けません")?;
    let field = package.read_initial()
        .context("SOFの initial.snc を読み込めません")?;
    let depths = field.coordinates().depth().to_vec();
    ensure!(!depths.is_empty(), "SOFの初期場に深度座標がありません");

    let terrain_package = SofPackage::open(&input_package_path)
        .context("SOFパッケージの地形データを開けません")?;
    let terrain = TerrainData::from_grid(
        terrain_package.read_terrain().context("SOFの terrain.snc を読み込めません")?,
    )?;

    println!("SOF読み込み完了: grid={}×{}×{} (lon × lat × depth)",
        field.velocity().grid().width(), field.velocity().grid().height(), depths.len());
    Ok(DatasetBundle { terrain, field, depths })
}

pub fn build_from_validated(config: &Config) -> Result<DatasetBundle> {
    println!("[1/7] 設定・入力データを読み込みます");
    let terrain_source = config.input.terrain_source.as_ref()
        .or(config.input.terrain_grid.as_ref())
        .ok_or_else(|| anyhow::anyhow!("input.terrain_source が必要です（terrain_grid は旧設定名です）"))?;
    let terrain_path = config.resolve(terrain_source);
    let mut terrain_reader = GridReader::open(&terrain_path)
        .with_context(|| format!("地形SOFを開けません: {}", terrain_path.display()))?;
    let terrain = TerrainData::from_grid(
        terrain_reader.read().context("地形SOFの読み込みに失敗しました")?,
    )?;

    let source = load_source_bundle(config)?;
    let source_coordinates = source.coordinates;
    ensure!(strictly_increasing(&source_coordinates.depth), "元データの深度は昇順である必要があります");
    ensure!(strictly_increasing_f64(&source_coordinates.latitude), "元データの緯度は昇順である必要があります");
    ensure!(strictly_increasing_f64(&source_coordinates.longitude), "元データの経度は昇順である必要があります");

    let source_width = source_coordinates.longitude.len();
    let source_height = source_coordinates.latitude.len();
    let source_plane = source_width * source_height;
    let source_count = source_plane * source_coordinates.depth.len();
    // Missing fields are explicit initial defaults, not recovered observations.
    let source_u = source.velocity_u.unwrap_or_else(|| { println!("      注意: 速度データなし。u=0 m/s で初期化します。"); vec![0.0; source_count] });
    let source_v = source.velocity_v.unwrap_or_else(|| { println!("      注意: 速度データなし。v=0 m/s で初期化します。"); vec![0.0; source_count] });
    let source_w = source.velocity_w.unwrap_or_else(|| { println!("      注意: 鉛直流速 w がありません。w=0 m/s で初期化します（観測値ではありません）。"); vec![0.0; source_count] });
    let source_t = source.temperature.unwrap_or_else(|| { println!("      注意: 水温データなし。初期値15°Cで初期化します（観測値ではありません）。"); vec![15.0; source_count] });
    let source_s = source.salinity.unwrap_or_else(|| { println!("      注意: 塩分データなし。初期値35 PSUで初期化します（観測値ではありません）。"); vec![35.0; source_count] });
    for (name, values) in [
        ("u", &source_u), ("v", &source_v), ("w", &source_w),
        ("temperature", &source_t), ("salinity", &source_s),
    ] {
        ensure!(values.len() == source_count,
            "{} データの要素数が不正です: {} != {} (depth={} × lat={} × lon={})",
            name, values.len(), source_count, source_coordinates.depth.len(), source_height, source_width);
    }

    let generated_grid = GridBuilder::build(&terrain, &source_coordinates, config.grid.depth_interval_m)?;
    let grid = generated_grid.grid;
    let coordinates = generated_grid.coordinates;
    let depths = generated_grid.depths;
    let interval = config.grid.depth_interval_m;
    let depth_count = depths.len();

    println!("[2/7] GridBuilder: {} × {} × {} (lon × lat × depth)", source_width, source_height, depth_count);
    println!("      深度 0..{:.0}m / Δz={:.0}m / dx={:.1}m / dy={:.1}m",
        generated_grid.max_sim_depth_m, interval, generated_grid.dx_m, generated_grid.dy_m);

    let mut u = expand_profile(&source_u, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, true);
    let mut v = expand_profile(&source_v, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, true);
    let mut t = expand_profile(&source_t, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, false);
    let mut s = expand_profile(&source_s, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, false);
    let mut w = expand_profile(&source_w, &source_coordinates.depth, &depths, source_plane,
        config.grid.deep_velocity_decay_scale_m, true);

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
    let source_description = if let Some(netcdf) = &config.input.netcdf {
        format!("Public NetCDF inputs: terrain={}, ocean={}", netcdf.terrain_file.display(), netcdf.ocean_file.display())
    } else {
        "Configured source bundle / raw velocity, temperature, salinity and bathymetry inputs".to_string()
    };
    metadata.set_source(source_description);
    metadata.set_history("Generated by apps/simulator dataset preparation");
    metadata.set_comment("Velocity below the deepest source layer is initialized using exponential attenuation; it is not source-observed deep data.");
    let boundary = derive_boundary_field(&velocity);
    let field = OceanField::new(
        coordinates, metadata, velocity, temperature, salinity, boundary,
    );
    println!("[5/7] 初期状態・境界条件・再開状態の元フィールドを構築しました。");

    let output_root = config.output_root();
    fs::create_dir_all(&output_root)
        .with_context(|| format!("出力先を作成できません: {}", output_root.display()))?;
    let input_package_path = output_root.join("input.sof");
    write_package(&input_package_path, &config.name, &field, &terrain, &depths, interval)?;
    println!("[6/7] SOFパッケージを生成: {}", input_package_path.display());

    // Use the serialized SOF as the actual simulation input.
    let package = SofPackageReader::open(&input_package_path)?;
    let field_from_sof = package.read_initial()
        .context("生成したSOFのinitial.sncを再読込できません")?;
    let terrain_package = SofPackage::open(&input_package_path)
        .context("生成したSOFパッケージを開けません")?;
    let terrain_from_sof = TerrainData::from_grid(
        terrain_package.read_terrain().context("生成したSOFのterrain.sncを再読込できません")?,
    )?;
    println!("[7/7] SOFを再読込して検証しました。");
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


/// Extracts the four edge faces from the generated velocity field for boundary.snc.
/// Values are copied from the source-derived initial state; they are not a forecast.
fn derive_boundary_field(source: &VelocityField) -> BoundaryField {
    let grid = *source.grid();
    let nx = grid.width();
    let ny = grid.height();
    let nz = grid.depth();
    let plane = nx * ny;
    let count = grid.len();
    let su = source.u().as_slice();
    let sv = source.v().as_slice();
    let sw = source.w().as_slice();
    let mut west = VelocityField::new(grid.clone());
    let mut east = VelocityField::new(grid.clone());
    let mut north = VelocityField::new(grid.clone());
    let mut south = VelocityField::new(grid);
    let mut edge_data: [[Vec<f32>; 3]; 4] =
        std::array::from_fn(|_| std::array::from_fn(|_| vec![0.0_f32; count]));
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let i = z * plane + y * nx + x;
                let values = [su[i], sv[i], sw[i]];
                if x == 0 { edge_data[0][0][i] = values[0]; edge_data[0][1][i] = values[1]; edge_data[0][2][i] = values[2]; }
                if x + 1 == nx { edge_data[1][0][i] = values[0]; edge_data[1][1][i] = values[1]; edge_data[1][2][i] = values[2]; }
                if y + 1 == ny { edge_data[2][0][i] = values[0]; edge_data[2][1][i] = values[1]; edge_data[2][2][i] = values[2]; }
                if y == 0 { edge_data[3][0][i] = values[0]; edge_data[3][1][i] = values[1]; edge_data[3][2][i] = values[2]; }
            }
        }
    }
    west.u_mut().replace(edge_data[0][0].clone());
    west.v_mut().replace(edge_data[0][1].clone());
    west.w_mut().replace(edge_data[0][2].clone());
    east.u_mut().replace(edge_data[1][0].clone());
    east.v_mut().replace(edge_data[1][1].clone());
    east.w_mut().replace(edge_data[1][2].clone());
    north.u_mut().replace(edge_data[2][0].clone());
    north.v_mut().replace(edge_data[2][1].clone());
    north.w_mut().replace(edge_data[2][2].clone());
    south.u_mut().replace(edge_data[3][0].clone());
    south.v_mut().replace(edge_data[3][1].clone());
    south.w_mut().replace(edge_data[3][2].clone());
    BoundaryField::new(west, east, north, south)
}

fn load_source_bundle(config: &Config) -> Result<SourceBundle> {
    if let Some(bundle_path) = &config.input.source_bundle {
        let path = config.resolve(bundle_path);
        if path.exists() {
            return SourceBundle::read(&path)
                .with_context(|| format!("ソースバンドルを読み込めません: {}", path.display()));
        }
        // A configured bundle is authoritative; do not silently fall back to stale raw files.
        anyhow::bail!("ソースバンドルがありません: {}。`cargo run -p simulator -- generate` を実行してください。", path.display());
    }
    let coordinates_path = config.input.source_coordinates.as_ref()
        .ok_or_else(|| anyhow::anyhow!("source_bundle または source_coordinates が必要です"))?;
    let coordinate_path = config.resolve(coordinates_path);
    let coordinate_bytes = fs::read(&coordinate_path)
        .with_context(|| format!("座標JSONを読めません: {}", coordinate_path.display()))?;
    let coordinates: SourceCoordinates = serde_json::from_slice(&coordinate_bytes)
        .with_context(|| format!("座標JSONが不正です: {}", coordinate_path.display()))?;
    let read_optional = |value: &Option<std::path::PathBuf>, name: &str| -> Result<Option<Vec<f32>>> {
        match value {
            Some(path) => Ok(Some(read_f32(config.resolve(path).as_path())?)),
            None => { println!("      注意: legacy 設定に {} がありません。デフォルト値を使用します。", name); Ok(None) }
        }
    };
    let u = read_optional(&config.input.velocity_u, "velocity_u")?;
    let v = read_optional(&config.input.velocity_v, "velocity_v")?;
    ensure!(u.is_some() == v.is_some(), "legacy 入力では u と v を両方指定してください");
    Ok(SourceBundle { coordinates, velocity_u: u, velocity_v: v, velocity_w: None,
        temperature: read_optional(&config.input.temperature, "temperature")?,
        salinity: read_optional(&config.input.salinity, "salinity")? })
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
fn strictly_increasing(values: &[f32]) -> bool { values.windows(2).all(|w| w[1] > w[0]) }
fn strictly_increasing_f64(values: &[f64]) -> bool { values.windows(2).all(|w| w[1] > w[0]) }
