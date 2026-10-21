mod config;
mod dataset;
mod engine;
mod render;
mod time_finder;

use anyhow::{Context, Result};
use config::Config;
use dataset::build;
use engine::SimulationEngine;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Suruga Ocean Simulator\n\nUsage: cargo run -p simulator -- [config.json] [--fresh|--resume] [--cpu|--metal]\n\n--fresh  Start from configured source data, ignoring checkpoints\n--resume Resume from the latest valid checkpoint\n--cpu    Use the CPU reference solver\n--metal  Use Apple Metal compute (default in config)\n");
        return Ok(());
    }
    let config_path = args.iter()
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| "apps/simulator/config/simulator.json".to_string());
    let mut config = Config::load(&config_path)
        .with_context(|| format!("設定ファイルを読み込めません: {config_path}"))?;
    if args.iter().any(|arg| arg == "--fresh") {
        config.output.resume_from_latest = false;
    }
    if args.iter().any(|arg| arg == "--resume") {
        config.output.resume_from_latest = true;
    }
    if args.iter().any(|arg| arg == "--cpu") {
        config.simulation.use_metal = false;
    }
    if args.iter().any(|arg| arg == "--metal") {
        config.simulation.use_metal = true;
    }

    println!("=== Suruga Ocean Simulator ===");
    println!("Compute backend: {}", if config.simulation.use_metal { "Apple Metal" } else { "CPU reference" });
    println!("Config: {}", config.resolve(&config_path).display());
    println!("Workspace: {}", config.workspace_root.display());
    println!("Resume latest checkpoint: {}", config.output.resume_from_latest);

    let bundle = build(&config)?;
    let render_enabled = config.render.enabled;
    let mut engine = SimulationEngine::new(config.clone(), bundle)?;
    if render_enabled {
        pollster::block_on(render::run(config, engine))?;
    } else {
        engine.run_to_completion()?;
    }
    Ok(())
}
