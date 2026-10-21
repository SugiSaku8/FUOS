mod config;
mod checkpoint;

use anyhow::{Context, Result, ensure};
use config::AppConfig;
use std::{env, fs, path::{Path, PathBuf}};

fn main() -> Result<()> {
    let config_path = env::args().nth(1).unwrap_or_else(|| "apps/simulator/config/simulator.json".into());
    let config = AppConfig::load(&config_path)?;
    let root = project_root(&config_path);

    println!("=== Suruga Ocean Simulator ===");
    println!("Config: {}", config_path);
    println!("Project: {}", config.project.name);
    println!("dt: {} s | duration: {} s | checkpoint: every {} steps",
        config.simulation.dt_seconds, config.simulation.duration_seconds, config.simulation.steps_per_checkpoint);

    // Phase 1: verify configured input sources before creating output artifacts.
    let terrain_path = resolve(&root, &config.terrain.source);
    let raw_root = resolve(&root, &config.ocean_data.root);
    let inputs = [
        ("terrain grid", terrain_path.clone()),
        ("u velocity", raw_root.join(&config.ocean_data.u)),
        ("v velocity", raw_root.join(&config.ocean_data.v)),
        ("temperature", raw_root.join(&config.ocean_data.temperature)),
        ("salinity", raw_root.join(&config.ocean_data.salinity)),
        ("coordinates", raw_root.join(&config.ocean_data.coordinates)),
        ("raw grid metadata", raw_root.join(&config.ocean_data.grid)),
    ];
    for (label, path) in &inputs {
        ensure!(path.is_file(), "{} が見つかりません: {}", label, path.display());
        println!("[input:ok] {} -> {}", label, path.display());
    }

    // Phase 2: create a run directory; never overwrite an existing run's results silently.
    let run_root = resolve(&root, &config.project.output_root);
    fs::create_dir_all(&run_root).with_context(|| format!("出力ディレクトリを作成できません: {}", run_root.display()))?;
    let checkpoint_dir = resolve(&root, &config.output.checkpoint_dir);
    fs::create_dir_all(&checkpoint_dir).with_context(|| format!("チェックポイント用ディレクトリを作成できません: {}", checkpoint_dir.display()))?;

    println!("\n[plan] 1/7 config と入力ファイルを確認");
    println!("[plan] 2/7 地形データから Grid を構築");
    println!("[plan] 3/7 海流データを整形し、地形とともに SOF へ出力");
    println!("[plan] 4/7 Metal を用いたシミュレーションを開始");
    println!("[plan] 5/7 レンダラーを同時起動し、共有スナップショットを受け取る");
    println!("[plan] 6/7 シミュレーション結果をリアルタイム描画");
    println!("[plan] 7/7 {} step ごとにチェックポイント、終了時に final SOF を保存", config.simulation.steps_per_checkpoint);

    // The loader/config bootstrap is intentionally separated from execution. This avoids
    // presenting static source currents as live solver output before the GPU/renderer bridge
    // and SOF field conversion are connected in the next integration step.
    println!("\n設定と入力ファイルの検証に成功しました。");
    println!("次の統合段階で、SOF生成・Metal solver・リアルタイムrenderer・チェックポイント書き込みを接続します。");
    println!("Run directory: {}", run_root.display());
    println!("Prepared SOF target: {}", resolve(&root, &config.project.prepared_sof).display());
    println!("Final SOF target: {}", resolve(&root, &config.output.final_sof).display());
    Ok(())
}

fn project_root(config_path: impl AsRef<Path>) -> PathBuf {
    let p = config_path.as_ref();
    // The default config is <repo>/apps/simulator/config/simulator.json.
    if p.is_absolute() { return p.parent().and_then(Path::parent).and_then(Path::parent).and_then(Path::parent).unwrap_or(Path::new(".")).to_path_buf(); }
    let normalized = p.to_string_lossy();
    if normalized.starts_with("apps/simulator/") { PathBuf::from(".") }
    else { p.parent().unwrap_or(Path::new(".")).to_path_buf() }
}
fn resolve(root: &Path, path: impl AsRef<Path>) -> PathBuf {
    let p = path.as_ref(); if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
}
