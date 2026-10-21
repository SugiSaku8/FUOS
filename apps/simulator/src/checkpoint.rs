use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{fs, io::Write, path::{Path, PathBuf}};

#[derive(Debug, Serialize)]
pub struct CheckpointMetadata {
    pub step: u64,
    pub simulation_time_seconds: f64,
    pub dt_seconds: f32,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub layout: String,
    pub fields: Vec<String>,
}

/// Atomic checkpoint writer. Each checkpoint is immutable and stored in its own directory.
/// The final SOF package is written separately at the end of the run.
pub struct CheckpointWriter { root: PathBuf }
impl CheckpointWriter {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).with_context(|| format!("checkpoint directory: {}", root.display()))?;
        Ok(Self { root })
    }

    pub fn save_velocity(&self, metadata: CheckpointMetadata, u: &[f32], v: &[f32], w: &[f32]) -> Result<PathBuf> {
        let expected = metadata.nx.checked_mul(metadata.ny).and_then(|n| n.checked_mul(metadata.nz))
            .context("checkpoint grid size overflow")?;
        ensure!(u.len() == expected && v.len() == expected && w.len() == expected,
            "checkpoint field size mismatch: expected {}, u={}, v={}, w={}", expected, u.len(), v.len(), w.len());
        ensure!(u.iter().chain(v).chain(w).all(|x| x.is_finite()), "checkpoint contains non-finite values");

        let name = format!("step_{:012}", metadata.step);
        let final_dir = self.root.join(&name);
        ensure!(!final_dir.exists(), "checkpoint already exists: {}", final_dir.display());
        let staging = self.root.join(format!(".{}.tmp", name));
        if staging.exists() { fs::remove_dir_all(&staging)?; }
        fs::create_dir_all(&staging)?;
        write_f32(&staging.join("u.bin"), u)?;
        write_f32(&staging.join("v.bin"), v)?;
        write_f32(&staging.join("w.bin"), w)?;
        fs::write(staging.join("metadata.json"), serde_json::to_vec_pretty(&metadata)?)?;
        fs::rename(&staging, &final_dir).with_context(|| format!("checkpoint commit failed: {}", final_dir.display()))?;
        Ok(final_dir)
    }
}
fn write_f32(path: &Path, values: &[f32]) -> Result<()> {
    let mut file = fs::File::create(path)?;
    for value in values { file.write_all(&value.to_le_bytes())?; }
    file.sync_all()?;
    Ok(())
}
