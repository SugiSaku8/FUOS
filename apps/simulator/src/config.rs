use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub project: ProjectConfig,
    pub terrain: TerrainConfig,
    pub ocean_data: OceanDataConfig,
    pub simulation: SimulationConfig,
    pub render: RenderConfig,
    pub output: OutputConfig,
}
#[derive(Debug, Deserialize)]
pub struct ProjectConfig { pub name: String, pub output_root: PathBuf, pub prepared_sof: PathBuf }
#[derive(Debug, Deserialize)]
pub struct TerrainConfig { pub source: PathBuf, pub format: String, pub vertical_scale: f32, pub contour_interval_m: f32 }
#[derive(Debug, Deserialize)]
pub struct OceanDataConfig {
    pub root: PathBuf, pub u: PathBuf, pub v: PathBuf, pub temperature: PathBuf, pub salinity: PathBuf,
    pub coordinates: PathBuf, pub grid: PathBuf, pub source_depths_m: Vec<f32>,
    pub target_depth_interval_m: f32, pub deep_extension: String, pub deep_decay_scale_m: f32,
    pub vertical_velocity_if_missing: f32,
}
#[derive(Debug, Deserialize)]
pub struct SimulationConfig { pub dt_seconds: f32, pub duration_seconds: f32, pub steps_per_checkpoint: u64, pub steps_per_render_update: u64, pub use_metal: bool }
#[derive(Debug, Deserialize)]
pub struct RenderConfig {
    pub enabled: bool, pub title: String, pub show_terrain: bool, pub show_streamlines: bool,
    pub show_particles: bool, pub show_depth_layers: bool, pub streamline_depth_interval_m: f32,
    pub vertical_scale: f32, pub target_fps: u32,
}
#[derive(Debug, Deserialize)]
pub struct OutputConfig { pub checkpoint_dir: PathBuf, pub final_sof: PathBuf, pub write_checkpoint_manifest: bool, pub keep_checkpoints: bool }

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).with_context(|| format!("設定ファイルを読めません: {}", path.display()))?;
        let config: Self = serde_json::from_str(&text).with_context(|| format!("設定JSONが不正です: {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<()> {
        if self.project.name.trim().is_empty() { bail!("project.name は空にできません"); }
        if !self.simulation.dt_seconds.is_finite() || self.simulation.dt_seconds <= 0.0 { bail!("simulation.dt_seconds は正の有限値にしてください"); }
        if !self.simulation.duration_seconds.is_finite() || self.simulation.duration_seconds <= 0.0 { bail!("simulation.duration_seconds は正の有限値にしてください"); }
        if self.simulation.steps_per_checkpoint == 0 { bail!("steps_per_checkpoint は1以上にしてください"); }
        if self.ocean_data.source_depths_m.is_empty() { bail!("source_depths_m が空です"); }
        if self.ocean_data.target_depth_interval_m <= 0.0 { bail!("target_depth_interval_m は正数にしてください"); }
        if self.ocean_data.deep_decay_scale_m <= 0.0 { bail!("deep_decay_scale_m は正数にしてください"); }
        if self.terrain.vertical_scale <= 0.0 || self.render.vertical_scale <= 0.0 { bail!("vertical_scale は正数にしてください"); }
        Ok(())
    }
    pub fn resolve_from(&self, config_path: impl AsRef<Path>, value: impl AsRef<Path>) -> PathBuf {
        let p = value.as_ref();
        if p.is_absolute() { return p.to_path_buf(); }
        let root = config_path.as_ref().parent().unwrap_or_else(|| Path::new("."));
        // Config paths are project-root relative, not config-directory relative.
        let project_root = root.parent().and_then(Path::parent).unwrap_or_else(|| Path::new("."));
        project_root.join(p)
    }
}
