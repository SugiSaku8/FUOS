use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{fs, path::{Path, PathBuf}};

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
    pub terrain_grid: PathBuf,
    pub source_coordinates: PathBuf,
    pub velocity_u: PathBuf,
    pub velocity_v: PathBuf,
    pub temperature: PathBuf,
    pub salinity: PathBuf,
}

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
