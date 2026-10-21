use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use s_field::OceanField;
use s_metal::{GpuOceanState, MetalContext, OceanSolver};
use s_sof::restart::{RestartReader, RestartWriter};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use crate::{
    config::Config,
    dataset::{DatasetBundle, TerrainData, write_result_package},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CheckpointMetadata {
    format: String,
    run_name: String,
    step: u64,
    simulation_time_seconds: f64,
    dt_seconds: f32,
    created_unix_seconds: u64,
    state_file: String,
}

pub struct SimulationEngine {
    config: Config,
    context: Option<Arc<MetalContext>>,
    solver: Option<OceanSolver>,
    state: Option<GpuOceanState>,
    cpu_solver: Option<s_cpu::CpuOceanSolver>,
    initial_field: OceanField,
    field: OceanField,
    terrain: TerrainData,
    depths: Vec<f32>,
    step: u64,
    time_seconds: f64,
    finished: bool,
    max_steps: u64,
}

impl SimulationEngine {
    pub fn new(config: Config, bundle: DatasetBundle) -> Result<Self> {
        println!("[4/7] {}計算エンジンを初期化します", if config.simulation.use_metal { "Metal" } else { "CPU" });
        let initial_field = bundle.field.clone();
        let mut field = bundle.field;
        let temperatures = initial_field.temperature().field().as_slice();
        let salinities = initial_field.salinity().field().as_slice();
        let wet_mask = temperatures.iter().zip(salinities).map(|(t, s)| {
            if *t == 0.0 && *s == 0.0 { 1.0_f32 } else { 0.0_f32 }
        }).collect::<Vec<_>>();
        let latitude = initial_field.coordinates().latitude().to_vec();
        let (context, solver, cpu_solver) = if config.simulation.use_metal {
            let context = Arc::new(MetalContext::new());
            let solver = OceanSolver::new_with_initial_boundary(
                context.clone(), config.simulation.kuroshio_inflow, initial_field.velocity(), &latitude, &wet_mask,
                config.simulation.horizontal_viscosity_m2_s,
                config.simulation.vertical_viscosity_m2_s,
                config.simulation.bottom_drag_coefficient,
            );
            (Some(context), Some(solver), None)
        } else {
            let solver = s_cpu::CpuOceanSolver::new(
                &initial_field, config.simulation.horizontal_viscosity_m2_s,
                config.simulation.vertical_viscosity_m2_s,
                config.simulation.bottom_drag_coefficient, config.simulation.kuroshio_inflow,
            );
            (None, None, Some(solver))
        };
        let mut step = 0_u64;
        let mut time_seconds = 0.0_f64;

        if config.output.resume_from_latest {
            if let Some((checkpoint_path, metadata)) = latest_checkpoint(&config.output_root())? {
                ensure!(metadata.run_name == config.name,
                    "チェックポイントの実行名 ({}) と現在の設定 ({}) が異なります", metadata.run_name, config.name);
                ensure!(metadata.simulation_time_seconds.is_finite() && metadata.simulation_time_seconds >= 0.0,
                    "チェックポイントのシミュレーション時刻が不正です");
                ensure!(
                    (metadata.dt_seconds - config.simulation.dt_seconds).abs() < f32::EPSILON,
                    "チェックポイントの時間刻み ({}) と現在の設定 ({}) が異なります",
                    metadata.dt_seconds,
                    config.simulation.dt_seconds
                );
                field = RestartReader::read(checkpoint_path.join(&metadata.state_file))
                    .with_context(|| format!("チェックポイントを読み込めません: {}", checkpoint_path.display()))?;
                step = metadata.step;
                time_seconds = metadata.simulation_time_seconds;
                println!(
                    "checkpoint resumed: {} (step={}, t={:.3}s)",
                    checkpoint_path.display(), step, time_seconds
                );
            } else {
                println!("resume_from_latest=true ですが、再開可能なチェックポイントがありません。初期状態から開始します。");
            }
        }

        let state = match context.as_ref() {
            Some(context) => Some(GpuOceanState::from_ocean(context, &field)),
            None => None,
        };
        let max_steps = (config.simulation.duration_seconds / config.simulation.dt_seconds as f64).ceil() as u64;
        let finished = step >= max_steps;
        let mut engine = Self {
            config,
            context,
            solver,
            state,
            cpu_solver,
            initial_field,
            field,
            terrain: bundle.terrain,
            depths: bundle.depths,
            step,
            time_seconds,
            finished: false,
            max_steps,
        };
        if finished {
            engine.save_final()?;
            engine.finished = true;
            println!("チェックポイントは指定された計算時間を既に完了しています。最終SOFを確認しました。");
        }
        Ok(engine)
    }

    /// High-resolution bathymetry used by the renderer.
    pub fn terrain(&self) -> &TerrainData { &self.terrain }

    /// Latest CPU-side ocean field (velocity is synchronized after each step).
    pub fn field(&self) -> &OceanField { &self.field }

    pub fn is_finished(&self) -> bool { self.finished }

    pub fn step(&self) -> u64 { self.step }

    pub fn time_seconds(&self) -> f64 { self.time_seconds }

    /// Clone a consistent CPU-side state for the time-finder history.
    pub fn snapshot(&mut self) -> OceanField {
        self.sync_cpu_field();
        self.field.clone()
    }

    /// Restore a previously captured state, rebuilding GPU buffers when Metal is active.
    pub fn restore_snapshot(&mut self, field: OceanField, step: u64, time_seconds: f64) {
        self.field = field;
        self.step = step;
        self.time_seconds = time_seconds.max(0.0);
        self.finished = self.step >= self.max_steps;
        self.state = self.context.as_ref().map(|context| GpuOceanState::from_ocean(context, &self.field));
    }

    pub fn advance(&mut self, requested_steps: usize) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        for _ in 0..requested_steps {
            if self.finished { break; }
            let dt = self.config.simulation.dt_seconds;
            let cfl = estimate_advection_cfl(&self.field, dt);
            ensure!(cfl.is_finite(), "CFL数が有限値ではありません: {cfl}");
            // Subcycle the momentum update when needed. This keeps the explicit
            // advection stage below a conservative CFL target without changing model time.
            let substeps = ((cfl / 0.45).ceil() as usize).max(1);
            ensure!(substeps <= 128,
                "時間刻みが大きすぎます (推定CFL={cfl:.3}, 必要サブステップ={substeps})。dt_secondsを小さくしてください");
            let sub_dt = dt / substeps as f32;
            for _ in 0..substeps {
                if self.config.simulation.use_metal {
                    let solver = self.solver.as_ref().expect("Metal backend solver missing");
                    let state = self.state.as_mut().expect("Metal backend state missing");
                    solver.step_with_tracers(
                        state, sub_dt,
                        self.field.temperature().field().as_slice(),
                        self.field.salinity().field().as_slice(),
                    );
                } else {
                    self.cpu_solver.as_ref().expect("CPU backend solver missing")
                        .step(&mut self.field, sub_dt);
                }
            }
            self.step += 1;
            self.time_seconds += dt as f64;

            // One GPU->CPU sync per model timestep; no readback inside CFL subcycling.
            self.sync_cpu_field();
            self.advance_scalars(dt);

            if self.config.output.save_checkpoints
                && self.step % self.config.simulation.steps_per_checkpoint == 0
            {
                self.save_checkpoint()?;
            }

            if self.step >= self.max_steps {
                self.save_final()?;
                self.finished = true;
                println!(
                    "[7/7] シミュレーション終了: step={}, t={:.3}s",
                    self.step, self.time_seconds
                );
            }
        }
        if self.step > 0 && self.step % 1000 == 0 {
            println!(
                "simulation: step={}, t={:.2}/{:.2}s | T={:.3}..{:.3}°C | S={:.3}..{:.3} PSU",
                self.step,
                self.time_seconds,
                self.config.simulation.duration_seconds,
                self.field.temperature().min().unwrap_or(0.0),
                self.field.temperature().max().unwrap_or(0.0),
                self.field.salinity().min().unwrap_or(0.0),
                self.field.salinity().max().unwrap_or(0.0),
            );
        }
        Ok(())
    }

    /// Saves the current state when the user closes the rendering window.
    pub fn finish_early(&mut self) -> Result<()> {
        if !self.finished {
            self.sync_cpu_field();
            self.save_final()?;
            self.finished = true;
            println!("シミュレーションを途中終了し、現在状態を保存しました (step={})", self.step);
        }
        Ok(())
    }

    pub fn run_to_completion(&mut self) -> Result<()> {
        while !self.finished {
            self.advance(self.config.simulation.steps_per_frame)?;
        }
        Ok(())
    }

    fn advance_scalars(&mut self, dt: f32) {
        let velocity = self.field.velocity();
        let u = velocity.u().time_slice(0).unwrap_or(&[]).to_vec();
        let v = velocity.v().time_slice(0).unwrap_or(&[]).to_vec();
        let w = velocity.w().time_slice(0).unwrap_or(&[]).to_vec();
        let grid = self.field.velocity().grid();
        let shape = grid.size(); // [time, depth, latitude, longitude]
        let nx = shape[3];
        let ny = shape[2];
        let nz = shape[1];
        let spacing = grid.spacing();
        let dx = spacing[3].max(1.0e-3);
        let dy = spacing[2].max(1.0e-3);
        let dz = spacing[1].max(1.0e-3);
        let coords = self.field.coordinates().clone();
        let terrain = self.terrain.clone();
        let diffusivity = self.config.simulation.scalar_diffusivity_m2_s;
        let temp = self.field.temperature().field().as_slice().to_vec();
        let salt = self.field.salinity().field().as_slice().to_vec();
        let next_temp = transport_scalar(&temp, &u, &v, &w, nx, ny, nz, dx, dy, dz, dt, diffusivity, &coords, &terrain);
        let next_salt = transport_scalar(&salt, &u, &v, &w, nx, ny, nz, dx, dy, dz, dt, diffusivity, &coords, &terrain);
        self.field.temperature_mut().field_mut().as_mut_slice().copy_from_slice(&next_temp);
        self.field.salinity_mut().field_mut().as_mut_slice().copy_from_slice(&next_salt);
    }

    fn save_checkpoint(&mut self) -> Result<()> {
        self.sync_cpu_field();
        let root = self.config.output_root().join("checkpoints");
        fs::create_dir_all(&root)?;
        let name = format!("step_{:012}", self.step);
        let staging = root.join(format!("{}.tmp", name));
        let destination = root.join(&name);
        if staging.exists() { fs::remove_dir_all(&staging)?; }
        fs::create_dir_all(&staging)?;
        let state_file = "restart.snc";
        RestartWriter::write(staging.join(state_file), &self.field)
            .context("チェックポイント状態を書き込めません")?;
        let metadata = CheckpointMetadata {
            format: "SOS-CHECKPOINT-1".to_string(),
            run_name: self.config.name.clone(),
            step: self.step,
            simulation_time_seconds: self.time_seconds,
            dt_seconds: self.config.simulation.dt_seconds,
            created_unix_seconds: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
            state_file: state_file.to_string(),
        };
        write_json_atomic(&staging.join("metadata.json"), &metadata)?;
        replace_directory(&staging, &destination)?;
        println!("checkpoint saved: {}", destination.display());
        Ok(())
    }

    fn save_final(&mut self) -> Result<()> {
        self.sync_cpu_field();
        if !self.config.output.save_final { return Ok(()); }
        let output = self.config.output_root().join("final.sof");
        if let Some(parent) = output.parent() { fs::create_dir_all(parent)?; }
        write_result_package(
            &output, &self.config.name, &self.initial_field, &self.field,
            &self.terrain, &self.depths, self.config.grid.depth_interval_m,
        ).context("最終SOFを書き込めません")?;
        println!("final SOF saved: {}", output.display());
        Ok(())
    }

    fn sync_cpu_field(&mut self) {
        let (Some(state), Some(context)) = (self.state.as_ref(), self.context.as_ref()) else {
            // CPU backend updates the OceanField in place.
            return;
        };
        let snapshot = state.velocity.snapshot(context);
        self.field.velocity_mut().u_mut().replace(snapshot.u);
        self.field.velocity_mut().v_mut().replace(snapshot.v);
        self.field.velocity_mut().w_mut().replace(snapshot.w);
    }
}

/// Estimate the advective CFL number on the current model grid.
fn estimate_advection_cfl(field: &OceanField, dt: f32) -> f32 {
    let velocity = field.velocity();
    let spacing = velocity.spacing();
    let dx = spacing[3].abs().max(1.0e-6);
    let dy = spacing[2].abs().max(1.0e-6);
    let dz = spacing[1].abs().max(1.0e-6);
    let u = velocity.u().time_slice(0).unwrap_or(&[]);
    let v = velocity.v().time_slice(0).unwrap_or(&[]);
    let w = velocity.w().time_slice(0).unwrap_or(&[]);
    u.iter().zip(v).zip(w).map(|((u,v),w)| {
        dt * (u.abs()/dx + v.abs()/dy + w.abs()/dz)
    }).fold(0.0_f32, f32::max)
}

/// Second-order TVD-limited scalar transport with SSP-RK2 time integration.
/// This improves numerical accuracy over first-order upwind/Euler while limiting
/// spurious oscillations near sharp fronts. It remains a simplified tracer model.
fn transport_scalar(
    old: &[f32], u: &[f32], v: &[f32], w: &[f32],
    nx: usize, ny: usize, nz: usize,
    dx: f32, dy: f32, dz: f32, dt: f32, diffusivity: f32,
    coords: &s_field::Coordinates, terrain: &TerrainData,
) -> Vec<f32> {
    if nx < 2 || ny < 2 || nz == 0 || old.is_empty() { return old.to_vec(); }
    let inv_h2 = 1.0/(dx*dx) + 1.0/(dy*dy) + 1.0/(dz*dz);
    let max_rate = u.iter().zip(v).zip(w).map(|((u,v),w)| u.abs()/dx+v.abs()/dy+w.abs()/dz)
        .fold(0.0_f32, f32::max);
    let advective_substeps = ((dt.max(0.0)*max_rate/0.45).ceil() as usize).max(1);
    let diffusive_substeps = if diffusivity > 0.0 {
        ((2.0*diffusivity*dt.max(0.0)*inv_h2/0.40).ceil() as usize).max(1)
    } else { 1 };
    let substeps = advective_substeps.max(diffusive_substeps).min(2048);
    let sub_dt = dt / substeps as f32;
    let stable_diffusivity = if diffusivity > 0.0 {
        diffusivity.min(0.40/(sub_dt.max(1.0e-20)*inv_h2))
    } else { 0.0 };
    let mut current = old.to_vec();
    for _ in 0..substeps {
        let rhs0 = scalar_rhs(&current,u,v,w,nx,ny,nz,dx,dy,dz,sub_dt,stable_diffusivity,coords,terrain);
        let predictor = current.iter().zip(&rhs0).map(|(q,r)| q+sub_dt*r).collect::<Vec<_>>();
        let rhs1 = scalar_rhs(&predictor,u,v,w,nx,ny,nz,dx,dy,dz,sub_dt,stable_diffusivity,coords,terrain);
        current = current.iter().zip(rhs0.iter().zip(&rhs1)).map(|(q,(a,b))| {
            let value=q+0.5*sub_dt*(a+b); if value.is_finite(){value}else{*q}
        }).collect();
    }
    current
}

fn scalar_rhs(
    q: &[f32], u: &[f32], v: &[f32], w: &[f32],
    nx: usize, ny: usize, nz: usize,
    dx: f32, dy: f32, dz: f32, _dt: f32, diffusivity: f32,
    coords: &s_field::Coordinates, terrain: &TerrainData,
) -> Vec<f32> {
    let plane = nx * ny;
    let mut rhs = vec![0.0_f32; q.len()];
    for z in 0..nz {
        let depth = coords.depth().get(z).copied().unwrap_or(z as f32 * dz);
        for y in 0..ny {
            let lat = coords.latitude_at(y).unwrap_or(0.0) as f32;
            let ty = nearest_index(&terrain.latitude, lat).min(terrain.height - 1);
            for x in 0..nx {
                let lon = coords.longitude_at(x).unwrap_or(0.0) as f32;
                let tx = nearest_index(&terrain.longitude, lon).min(terrain.width - 1);
                let seabed = terrain.bathymetry[ty * terrain.width + tx];
                let i = z * plane + y * nx + x;
                if seabed >= 0.0 || depth > -seabed || !q[i].is_finite() { continue; }

                let xl = if x > 0 { i - 1 } else { i };
                let xr = if x + 1 < nx { i + 1 } else { i };
                let yd = if y > 0 { i - nx } else { i };
                let yu = if y + 1 < ny { i + nx } else { i };
                let zb = if z > 0 { i - plane } else { i };
                let za = if z + 1 < nz { i + plane } else { i };

                // Monotonized central slopes: second-order in smooth regions,
                // reduced to first order near extrema to suppress overshoot.
                let sx = if x > 0 && x + 1 < nx {
                    minmod((q[i] - q[xl]) / dx, (q[xr] - q[i]) / dx)
                } else { 0.0 };
                let sy = if y > 0 && y + 1 < ny {
                    minmod((q[i] - q[yd]) / dy, (q[yu] - q[i]) / dy)
                } else { 0.0 };
                let sz = if z > 0 && z + 1 < nz {
                    minmod((q[i] - q[zb]) / dz, (q[za] - q[i]) / dz)
                } else { 0.0 };

                let ux = u.get(i).copied().unwrap_or(0.0);
                let vy = v.get(i).copied().unwrap_or(0.0);
                // Positive w is upward in the renderer; depth coordinates increase downward.
                let wz = -w.get(i).copied().unwrap_or(0.0);
                let laplacian = (q[xl] - 2.0 * q[i] + q[xr]) / (dx * dx)
                    + (q[yd] - 2.0 * q[i] + q[yu]) / (dy * dy)
                    + (q[zb] - 2.0 * q[i] + q[za]) / (dz * dz);
                rhs[i] = -(ux * sx + vy * sy + wz * sz) + diffusivity * laplacian;
                if !rhs[i].is_finite() { rhs[i] = 0.0; }
            }
        }
    }
    rhs
}

#[inline]
fn minmod(a: f32, b: f32) -> f32 {
    if a * b <= 0.0 { 0.0 } else if a.abs() < b.abs() { a } else { b }
}

fn latest_checkpoint(root: &Path) -> Result<Option<(PathBuf, CheckpointMetadata)>> {
    let checkpoint_root = root.join("checkpoints");
    if !checkpoint_root.is_dir() { return Ok(None); }
    let mut candidates = Vec::new();
    for entry in fs::read_dir(&checkpoint_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() { continue; }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else { continue; };
        if name.ends_with(".tmp") || !name.starts_with("step_") { continue; }
        let metadata_path = path.join("metadata.json");
        let Ok(bytes) = fs::read(&metadata_path) else { continue; };
        let Ok(metadata) = serde_json::from_slice::<CheckpointMetadata>(&bytes) else { continue; };
        if metadata.format != "SOS-CHECKPOINT-1" || !path.join(&metadata.state_file).is_file() { continue; }
        candidates.push((metadata.step, path, metadata));
    }
    candidates.sort_by_key(|v| v.0);
    Ok(candidates.pop().map(|(_, path, metadata)| (path, metadata)))
}

fn replace_directory(staging: &Path, destination: &Path) -> Result<()> {
    let backup = destination.with_extension("previous");
    if backup.exists() { fs::remove_dir_all(&backup)?; }
    if destination.exists() { fs::rename(destination, &backup).with_context(|| format!("既存出力を退避できません: {}", destination.display()))?; }
    if let Err(error) = fs::rename(staging, destination) {
        if backup.exists() && !destination.exists() { let _ = fs::rename(&backup, destination); }
        return Err(error).with_context(|| format!("出力ディレクトリを確定できません: {}", destination.display()));
    }
    if backup.exists() { fs::remove_dir_all(backup)?; }
    Ok(())
}

fn write_json_atomic(path: &Path, value: &impl Serialize) -> Result<()> {
    let staging = path.with_extension("json.tmp");
    fs::write(&staging, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&staging, path).with_context(|| format!("JSONを確定できません: {}", path.display()))?;
    Ok(())
}

fn nearest_index(values: &[f32], value: f32) -> usize {
    values.iter().enumerate()
        .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
        .map(|(i, _)| i).unwrap_or(0)
}


#[cfg(test)]
mod tests {
    use super::*;
    use s_field::Grid4D;

    #[test]
    fn constant_scalar_is_preserved_by_advection() {
        let grid = Grid4D::new([1, 2, 2, 2], [1.0, 1.0, 1.0, 1.0]);
        let coordinates = s_field::Coordinates::new(
            grid,
            vec![0.0, 1.0],
            vec![0.0, 1.0],
            vec![0.0, 1.0],
            vec![0.0],
        );
        let terrain = TerrainData {
            longitude: vec![0.0, 1.0],
            latitude: vec![0.0, 1.0],
            bathymetry: vec![-100.0; 4],
            width: 2,
            height: 2,
        };
        let old = vec![12.5; 8];
        let u = vec![0.2; 8];
        let v = vec![-0.1; 8];
        let w = vec![0.0; 8];
        let result = transport_scalar(
            &old, &u, &v, &w, 2, 2, 2, 1.0, 1.0, 1.0, 0.1, 0.0,
            &coordinates, &terrain,
        );
        assert!(result.iter().all(|value| (*value - 12.5).abs() < 1.0e-6));
    }

    #[test]
    fn minmod_limits_slopes_and_preserves_smooth_direction() {
        assert_eq!(minmod(2.0, 3.0), 2.0);
        assert_eq!(minmod(-2.0, -3.0), -2.0);
        assert_eq!(minmod(2.0, -3.0), 0.0);
        assert_eq!(minmod(0.0, 3.0), 0.0);
    }

    #[test]
    fn latest_checkpoint_ignores_temporary_directories() {
        let root = std::env::temp_dir().join(format!(
            "sos-checkpoint-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_nanos(),
        ));
        let checkpoint_root = root.join("checkpoints");
        fs::create_dir_all(checkpoint_root.join("step_000000000010.tmp")).unwrap();
        assert!(latest_checkpoint(&root).unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
