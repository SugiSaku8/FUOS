use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub input: InputConfig,
    pub grid: GridConfig,
    pub simulation: SimulationConfig,
    pub render: RenderConfig,
    pub output: OutputConfig,
    #[serde(skip)]
    pub workspace_root: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InputConfig {
    /// Preferred explicit name for the terrain source (not the generated solver grid).
    #[serde(default)]
    pub terrain_source: Option<PathBuf>,
    /// Legacy config key retained for existing users.
    #[serde(default)]
    pub terrain_grid: Option<PathBuf>,
    /// Preferred compact source file containing coordinates and selected physical fields.
    #[serde(default)]
    pub source_bundle: Option<PathBuf>,
    /// Legacy paths are retained for backward compatibility and one-time bundling.
    #[serde(default)]
    pub source_coordinates: Option<PathBuf>,
    #[serde(default)]
    pub velocity_u: Option<PathBuf>,
    #[serde(default)]
    pub velocity_v: Option<PathBuf>,
    #[serde(default)]
    pub temperature: Option<PathBuf>,
    #[serde(default)]
    pub salinity: Option<PathBuf>,
    /// Public NetCDF input configuration (GEBCO terrain + HYCOM/ocean fields).
    #[serde(default)]
    pub netcdf: Option<NetcdfInputConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetcdfInputConfig {
    /// Public bathymetry/terrain NetCDF or GeoTIFF (for example GEBCO).
    pub terrain_file: PathBuf,
    /// Public ocean physical-field NetCDF (for example HYCOM).
    pub ocean_file: PathBuf,
    #[serde(default = "default_prepared_dir")]
    pub prepared_dir: PathBuf,
    #[serde(default)]
    pub region: NetcdfRegionConfig,
    #[serde(default)]
    pub time_index: usize,
    /// Maximum depth (m, positive down) retrieved by `get --hycom`.
    #[serde(default = "default_hycom_max_depth")]
    pub hycom_max_depth_m: f64,
    /// Optional HYCOM OPeNDAP endpoint override.
    #[serde(default)]
    pub hycom_url: Option<String>,
    /// Explicit NetCDF variable/coordinate overrides. Keys are documented in tools/netcdf/README.md.
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetcdfRegionConfig {
    #[serde(default = "default_lon_min")]
    pub longitude_min: f64,
    #[serde(default = "default_lon_max")]
    pub longitude_max: f64,
    #[serde(default = "default_lat_min")]
    pub latitude_min: f64,
    #[serde(default = "default_lat_max")]
    pub latitude_max: f64,
}

impl Default for NetcdfRegionConfig {
    fn default() -> Self {
        Self { longitude_min: default_lon_min(), longitude_max: default_lon_max(),
            latitude_min: default_lat_min(), latitude_max: default_lat_max() }
    }
}

fn default_prepared_dir() -> PathBuf { PathBuf::from("runs/suruga_bay/preprocessed") }
fn default_hycom_max_depth() -> f64 { 2000.0 }
fn default_lon_min() -> f64 { 137.5 }
fn default_lon_max() -> f64 { 139.5 }
fn default_lat_min() -> f64 { 33.5 }
fn default_lat_max() -> f64 { 35.5 }

#[derive(Clone, Debug, Deserialize)]
pub struct GridConfig {
    pub depth_interval_m: f32,
    pub deep_velocity_decay_scale_m: f32,
    pub terrain_vertical_scale: f32,
    pub world_scale: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationConfig {
    pub dt_seconds: f32,
    pub duration_seconds: f64,
    pub steps_per_frame: usize,
    pub steps_per_checkpoint: u64,
    pub use_metal: bool,
    pub kuroshio_inflow: bool,
    #[serde(default = "default_scalar_diffusivity")]
    pub scalar_diffusivity_m2_s: f32,
    #[serde(default = "default_horizontal_viscosity")]
    pub horizontal_viscosity_m2_s: f32,
    #[serde(default = "default_vertical_viscosity")]
    pub vertical_viscosity_m2_s: f32,
    #[serde(default = "default_bottom_drag")]
    pub bottom_drag_coefficient: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RenderConfig {
    pub enabled: bool,
    pub show_ocean_surface: bool,
    pub show_bathymetry_contours: bool,
    pub contour_interval_m: f32,
    pub particle_trail_points: usize,
    pub particle_speed_multiplier: f32,
    pub seed_stride_x: usize,
    pub seed_stride_y: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OutputConfig {
    pub root: PathBuf,
    pub save_checkpoints: bool,
    pub save_final: bool,
    #[serde(default)]
    pub resume_from_latest: bool,
}

fn default_scalar_diffusivity() -> f32 { 1.0e-5 }
fn default_horizontal_viscosity() -> f32 { 10.0 }
fn default_vertical_viscosity() -> f32 { 1.0e-4 }
fn default_bottom_drag() -> f32 { 2.5e-3 }

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let requested = path.as_ref();
        let path = if requested.is_absolute() {
            requested.to_path_buf()
        } else {
            std::env::current_dir()?.join(requested)
        };
        let text = fs::read_to_string(&path)
            .with_context(|| format!("設定ファイルを読み込めません: {}", path.display()))?;
        let mut config: Self = serde_json::from_str(&text)
            .with_context(|| format!("設定JSONを解析できません: {}", path.display()))?;
        let start = path.parent().unwrap_or(Path::new("."));
        config.workspace_root = find_workspace_root(start)
            .or_else(|| find_workspace_root(&std::env::current_dir().ok()?))
            .ok_or_else(|| anyhow::anyhow!("Cargo workspace のルートを特定できません"))?;
        config.validate()?;
        Ok(config)
    }

    pub fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() { path.to_path_buf() } else { self.workspace_root.join(path) }
    }

    pub fn output_root(&self) -> PathBuf {
        self.resolve(&self.output.root)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(!self.name.trim().is_empty(), "name は空にできません");
        if let Some(netcdf) = &self.input.netcdf {
            ensure!(netcdf.region.longitude_min.is_finite() && netcdf.region.longitude_max.is_finite()
                && netcdf.region.latitude_min.is_finite() && netcdf.region.latitude_max.is_finite(),
                "input.netcdf.region は有限値で指定してください");
            ensure!(netcdf.region.longitude_min < netcdf.region.longitude_max,
                "input.netcdf.region.longitude_min は longitude_max より小さくしてください");
            ensure!(netcdf.region.latitude_min < netcdf.region.latitude_max,
                "input.netcdf.region.latitude_min は latitude_max より小さくしてください");
            ensure!(netcdf.hycom_max_depth_m.is_finite() && netcdf.hycom_max_depth_m > 0.0,
                "input.netcdf.hycom_max_depth_m は正の有限値にしてください");
            if let Some(url) = &netcdf.hycom_url {
                ensure!(url.starts_with("https://") || url.starts_with("http://"),
                    "input.netcdf.hycom_url は http(s) URL にしてください");
            }
            for (key, value) in &netcdf.variables {
                ensure!(!key.trim().is_empty() && !value.trim().is_empty(),
                    "input.netcdf.variables のキー・値は空にできません");
            }
        }
        ensure!(self.grid.depth_interval_m.is_finite() && self.grid.depth_interval_m > 0.0,
            "grid.depth_interval_m は正の有限値にしてください");
        ensure!(self.grid.deep_velocity_decay_scale_m.is_finite() && self.grid.deep_velocity_decay_scale_m > 0.0,
            "grid.deep_velocity_decay_scale_m は正の有限値にしてください");
        ensure!(self.grid.terrain_vertical_scale.is_finite() && self.grid.terrain_vertical_scale > 0.0,
            "grid.terrain_vertical_scale は正の有限値にしてください");
        ensure!(self.grid.world_scale.is_finite() && self.grid.world_scale > 0.0,
            "grid.world_scale は正の有限値にしてください");
        ensure!(self.simulation.dt_seconds.is_finite() && self.simulation.dt_seconds > 0.0,
            "simulation.dt_seconds は正の有限値にしてください");
        ensure!(self.simulation.duration_seconds.is_finite() && self.simulation.duration_seconds > 0.0,
            "simulation.duration_seconds は正の有限値にしてください");
        ensure!(self.simulation.steps_per_frame > 0, "simulation.steps_per_frame は1以上にしてください");
        ensure!(self.simulation.steps_per_checkpoint > 0, "simulation.steps_per_checkpoint は1以上にしてください");
        ensure!(self.simulation.scalar_diffusivity_m2_s.is_finite() && self.simulation.scalar_diffusivity_m2_s >= 0.0,
            "simulation.scalar_diffusivity_m2_s は0以上の有限値にしてください");
        ensure!(self.simulation.horizontal_viscosity_m2_s.is_finite() && self.simulation.horizontal_viscosity_m2_s >= 0.0,
            "simulation.horizontal_viscosity_m2_s は0以上の有限値にしてください");
        ensure!(self.simulation.vertical_viscosity_m2_s.is_finite() && self.simulation.vertical_viscosity_m2_s >= 0.0,
            "simulation.vertical_viscosity_m2_s は0以上の有限値にしてください");
        ensure!(self.simulation.bottom_drag_coefficient.is_finite() && (0.0..=0.1).contains(&self.simulation.bottom_drag_coefficient),
            "simulation.bottom_drag_coefficient は0..=0.1の有限値にしてください");
        ensure!(self.render.contour_interval_m.is_finite() && self.render.contour_interval_m > 0.0,
            "render.contour_interval_m は正の有限値にしてください");
        ensure!(self.render.particle_trail_points >= 2, "render.particle_trail_points は2以上にしてください");
        ensure!(self.render.seed_stride_x > 0 && self.render.seed_stride_y > 0,
            "render.seed_stride_x / seed_stride_y は1以上にしてください");
        Ok(())
    }
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let manifest = ancestor.join("Cargo.toml");
        if let Ok(text) = fs::read_to_string(manifest) {
            if text.contains("[workspace]") {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}
