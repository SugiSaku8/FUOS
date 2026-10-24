mod config;
mod dataset;
mod diagnostics;
mod pipeline;
mod engine;
mod generator_cli;
mod grid_builder;
mod source_bundle;
mod render;
mod time_finder;

use anyhow::{Context, Result};
use config::Config;
use pipeline::build;
use engine::SimulationEngine;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("generate") {
        let config_arg = args.iter().skip(1).find(|a| !a.starts_with('-')).map(String::as_str);
        return generator_cli::run(config_arg);
    }
    if args.first().map(String::as_str) == Some("diagnose") {
        let rest = args.iter().skip(1).collect::<Vec<_>>();
        let json_value_index = rest.iter().position(|a| a.as_str() == "--json").map(|i| i + 1);
        let config_arg = rest.iter().enumerate()
            .find(|(i, a)| !a.starts_with('-') && Some(*i) != json_value_index)
            .map(|(_, s)| s.as_str())
            .unwrap_or("apps/simulator/config/simulator.json");
        let config = Config::load(config_arg)
            .with_context(|| format!("設定ファイルを読み込めません: {config_arg}"))?;
        let report = pipeline::diagnose_inputs(&config)?;
        if let Some(index) = rest.iter().position(|a| a.as_str() == "--json") {
            let out = rest.get(index + 1).ok_or_else(|| anyhow::anyhow!("--json の後に出力パスが必要です"))?;
            let report_path = config.resolve(out.as_str());
            diagnostics::write_json(&report, &report_path)?;
            println!("診断JSONを書き込みました: {}", report_path.display());
        }
        anyhow::ensure!(!report.has_errors(), "元データ診断に ERROR があります。レポートを修正してから再実行してください。");
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Suruga Ocean Simulator\n\nUsage:\n  cargo run -p simulator -- generate [config.json]\n  cargo run -p simulator -- diagnose [config.json] [--json report.json]\n  cargo run -p simulator -- [config.json] [--fresh|--resume] [--cpu|--metal]\n\nCommands and options:\n  generate  Y/N wizard: diagnose, bundle source data, generate Grid and SOF inputs\n  diagnose  Inspect source files, axes, field ranges, and terrain coverage\n  --fresh   Start from configured source data, ignoring checkpoints\n  --resume  Resume from the latest valid checkpoint\n  --cpu     Use the CPU reference solver\n  --metal   Use Apple Metal compute (default in config)\n");
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
