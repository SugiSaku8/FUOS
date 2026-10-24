mod config;
mod dataset;
mod diagnostics;
mod pipeline;
mod netcdf_pipeline;
mod engine;
mod generator_cli;
mod grid_builder;
mod source_bundle;
mod render;
mod time_finder;

use anyhow::{Context, Result};
use config::Config;
use dataset::load_existing_sof;
use engine::SimulationEngine;
use std::io::{self, Write};

const DEFAULT_CONFIG: &str = "apps/simulator/config/simulator.json";

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let config_arg = args.iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str)
        .unwrap_or(DEFAULT_CONFIG);

    if args.first().map(String::as_str) == Some("get") {
        if !args.iter().any(|arg| arg == "--hycom") {
            anyhow::bail!("get では --hycom を指定してください。例: cargo run -p simulator -- get --hycom");
        }
        let config_arg = args.iter().skip(1).find(|arg| !arg.starts_with('-')).map(String::as_str).unwrap_or(DEFAULT_CONFIG);
        let force = args.iter().any(|arg| arg == "--force");
        return get_hycom(config_arg, force);
    }
    if args.first().map(String::as_str) == Some("generate") {
        return generator_cli::run(args.iter().skip(1).find(|a| !a.starts_with('-')).map(String::as_str));
    }
    if args.first().map(String::as_str) == Some("diagnose") {
        return run_diagnose(&args[1..]);
    }

    if args.iter().any(|arg| arg == "--y") {
        if confirm("すべての処理を Y で実行しますか？")? {
            return run_all(config_arg, true);
        }
        println!("N が選択されました。--y を指定しなかった場合と同じく、シミュレーションのみを実行します。\n");
        return run_simulation(config_arg, &args);
    }
    if args.iter().any(|arg| arg == "--all") {
        return run_all(config_arg, false);
    }
    if args.iter().any(|arg| arg == "--simulation") || args.is_empty() || args.iter().all(|a| a.starts_with('-')) {
        return run_simulation(config_arg, &args);
    }

    // Backward compatibility: a config path plus solver flags means simulation-only.
    run_simulation(config_arg, &args)
}

fn run_all(config_arg: &str, auto_yes: bool) -> Result<()> {
    println!("=== SOS 一括実行: get(if needed) → generate → diagnose → simulation ===\n");
    ensure_hycom_if_only_terrain_exists(config_arg)?;
    generator_cli::run_with_auto_yes(Some(config_arg), auto_yes)?;
    run_diagnose(&[config_arg.to_string()])?;
    run_simulation(config_arg, &[])?;
    println!("\n=== SOS 一括実行完了 ===");
    Ok(())
}

fn get_hycom(config_arg: &str, force: bool) -> Result<()> {
    let config = Config::load(config_arg)
        .with_context(|| format!("設定ファイルを読み込めません: {config_arg}"))?;
    println!("=== HYCOM 自動取得（Rust/curl） ===");
    println!("保存先: {}", config.input.netcdf.as_ref()
        .map(|nc| config.resolve(&nc.ocean_file).display().to_string())
        .unwrap_or_else(|| "input.netcdf.ocean_file 未設定".to_string()));
    netcdf_pipeline::download_hycom(&config, force)
}

/// In --all, if terrain is present but the configured ocean data is missing, fetch HYCOM.
fn ensure_hycom_if_only_terrain_exists(config_arg: &str) -> Result<()> {
    let config = Config::load(config_arg)
        .with_context(|| format!("設定ファイルを読み込めません: {config_arg}"))?;
    let Some(netcdf) = config.input.netcdf.as_ref() else { return Ok(()); };
    let terrain = config.resolve(&netcdf.terrain_file);
    let ocean = config.resolve(&netcdf.ocean_file);
    if terrain.is_file() && !ocean.is_file() {
        println!("[AUTO] 地形データはありますが海洋入力がありません。HYCOMを自動取得します。\n");
        get_hycom(config_arg, false)?;
    }
    Ok(())
}

fn run_diagnose(args: &[String]) -> Result<()> {
    let json_value_index = args.iter().position(|a| a == "--json").map(|i| i + 1);
    let config_arg = args.iter().enumerate()
        .find(|(i, a)| !a.starts_with('-') && Some(*i) != json_value_index)
        .map(|(_, s)| s.as_str())
        .unwrap_or(DEFAULT_CONFIG);
    let config = Config::load(config_arg)
        .with_context(|| format!("設定ファイルを読み込めません: {config_arg}"))?;
    let resolved_config_path = if std::path::Path::new(config_arg).is_absolute() {
        std::path::PathBuf::from(config_arg)
    } else { std::env::current_dir()?.join(config_arg) };
    let preparation_error = netcdf_pipeline::prepare(&config, &resolved_config_path).err();
    let report = pipeline::diagnose_inputs(&config)?;
    if let Some(index) = args.iter().position(|a| a == "--json") {
        let out = args.get(index + 1)
            .ok_or_else(|| anyhow::anyhow!("--json の後に出力パスが必要です"))?;
        let report_path = config.resolve(out);
        diagnostics::write_json(&report, &report_path)?;
        println!("診断JSONを書き込みました: {}", report_path.display());
    }
    if let Some(error) = preparation_error {
        anyhow::bail!("NetCDF/GeoTIFF入力の前処理に失敗しました。diagnostics.json も確認してください: {error:#}");
    }
    anyhow::ensure!(!report.has_errors(), "元データ診断に ERROR があります。レポートを修正してから再実行してください。");
    Ok(())
}

fn run_simulation(config_path: &str, args: &[String]) -> Result<()> {
    let mut config = Config::load(config_path)
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
    println!("Mode: simulation only (existing SOF)");
    println!("Compute backend: {}", if config.simulation.use_metal { "Apple Metal" } else { "CPU reference" });
    println!("Config: {}", config.resolve(config_path).display());
    println!("Workspace: {}", config.workspace_root.display());
    println!("Resume latest checkpoint: {}", config.output.resume_from_latest);

    let bundle = load_existing_sof(&config)?;
    let render_enabled = config.render.enabled;
    let engine = SimulationEngine::new(config.clone(), bundle)?;
    if render_enabled {
        pollster::block_on(render::run(config, engine))?;
    } else {
        let mut engine = engine;
        engine.run_to_completion()?;
    }
    Ok(())
}

fn confirm(question: &str) -> Result<bool> {
    loop {
        print!("{} [y/N] ", question);
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 {
            return Ok(false);
        }
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "" | "n" | "no" => return Ok(false),
            _ => println!("Y または N で入力してください。"),
        }
    }
}

fn print_help() {
    println!("Suruga Ocean Simulator\n\nUsage:\n  cargo run -p simulator -- generate [config.json]\n  cargo run -p simulator -- diagnose [config.json] [--json report.json]\n  cargo run -p simulator -- get --hycom [config.json] [--force]\n  cargo run -p simulator -- --simulation [config.json] [--fresh|--resume] [--cpu|--metal]\n  cargo run -p simulator -- --all [config.json]\n  cargo run -p simulator -- --y [config.json]\n\nCommands:\n  generate      Generate the source bundle and SOF input package (interactive)\n  get --hycom   Download the Suruga Bay HYCOM subset to the configured data/raw path\n  diagnose      Diagnose configured input data and write diagnostics.json\n  --simulation  Run only from an existing input.sof; does not generate inputs\n  --all         Run generate, diagnose, and simulation in sequence\n  --y           Ask once; Y auto-answers all generator prompts, N runs simulation only\n\nSimulation options:\n  --fresh       Ignore checkpoints and start from SOF initial state\n  --resume      Resume from the latest valid checkpoint\n  --cpu         Use the CPU reference solver\n  --metal       Use Apple Metal compute\n");
}
