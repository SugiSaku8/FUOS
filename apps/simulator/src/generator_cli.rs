//! Interactive Y/N wizard for creating the compact source bundle and SOF input package.
use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

use crate::{config::Config, source_bundle::SourceBundle};

pub fn run(config_arg: Option<&str>) -> Result<()> {
    let config_arg = config_arg.unwrap_or("apps/simulator/config/simulator.json");
    let mut config = Config::load(config_arg)
        .with_context(|| format!("設定ファイルを読み込めません: {config_arg}"))?;
    let workspace = config.workspace_root.clone();
    let config_path = if Path::new(config_arg).is_absolute() {
        PathBuf::from(config_arg)
    } else {
        workspace.join(config_arg)
    };

    println!("\n============================================");
    println!(" Suruga Ocean Simulator / Data Generator");
    println!("============================================\n");
    println!("入力を1つのソースバンドルにまとめ、Grid・各物理場・SOFパッケージを生成します。\n");
    let terrain_source = config.input.terrain_source.as_ref().or(config.input.terrain_grid.as_ref())
        .ok_or_else(|| anyhow::anyhow!("input.terrain_source が必要です（旧設定 terrain_grid も利用できます）"))?;
    println!("地形データは独立した入力として維持します: {}", config.resolve(terrain_source).display());

    let default_bundle = config.input.source_bundle.clone()
        .unwrap_or_else(|| PathBuf::from("crates/s-builder/suruga_raw/ocean_source.sosdata"));
    let bundle_text = ask_text("出力するソースバンドルのパス", &default_bundle.display().to_string())?;
    let bundle_config_path = PathBuf::from(bundle_text);
    let bundle_path = resolve(&workspace, &bundle_config_path);

    let create_bundle = ask_yes_no("既存の raw データからソースバンドルを作成/更新しますか？", true)?;
    if create_bundle {
        let raw_default = "crates/s-builder/suruga_raw";
        let raw_text = ask_text("raw データのディレクトリ（coordinates.json, *.bin）", raw_default)?;
        let raw_dir = resolve(&workspace, Path::new(&raw_text));
        let include_velocity = ask_yes_no("海流速度 u/v をソースに含めますか？", true)?;
        let include_temperature = ask_yes_no("水温をソースに含めますか？", true)?;
        let include_salinity = ask_yes_no("塩分をソースに含めますか？", true)?;
        let bundle = SourceBundle::from_legacy_dir(&raw_dir, include_velocity, include_temperature, include_salinity)
            .with_context(|| format!("raw データをまとめられません: {}", raw_dir.display()))?;
        bundle.write(&bundle_path)?;
        println!("\nOK: {} にまとめました", bundle_path.display());
    } else {
        let existing = SourceBundle::read(&bundle_path)
            .with_context(|| format!("既存バンドルを読み込めません: {}", bundle_path.display()))?;
        println!("既存バンドルを検証しました: depth={} × lat={} × lon={}",
            existing.coordinates.depth.len(), existing.coordinates.latitude.len(), existing.coordinates.longitude.len());
    }

    // Persist the new single-file input contract and remove obsolete per-variable paths.
    let mut json: Value = serde_json::from_slice(&fs::read(&config_path)
        .with_context(|| format!("設定ファイルを読めません: {}", config_path.display()))?)?;
    let input = json.get_mut("input").and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("設定ファイルに input オブジェクトがありません"))?;
    input.insert("source_bundle".to_string(), Value::String(bundle_config_path.to_string_lossy().to_string()));
    if !input.contains_key("terrain_source") {
        if let Some(old_terrain) = input.remove("terrain_grid") {
            input.insert("terrain_source".to_string(), old_terrain);
        }
    }
    for key in ["source_coordinates", "velocity_u", "velocity_v", "temperature", "salinity"] {
        input.remove(key);
    }
    fs::write(&config_path, serde_json::to_vec_pretty(&json)?)
        .with_context(|| format!("設定ファイルを更新できません: {}", config_path.display()))?;
    println!("設定を更新しました: {}", config_path.display());

    // Diagnose the actual persisted inputs before any derived data is generated.
    config = Config::load(&config_path)?;
    let diagnostic = crate::pipeline::diagnose_inputs(&config)?;
    anyhow::ensure!(!diagnostic.has_errors(), "元データ診断に ERROR があります。SOF生成を中止しました。入力を修正して再実行してください。");
    if diagnostic.has_warnings() && !ask_yes_no("警告があります。内容を確認したうえで生成を続行しますか？", false)? {
        println!("診断結果を確認して生成を中止しました。入力データは変更されていません（ソースバンドル/設定の更新を除く）。");
        return Ok(());
    }
    println!("\n[Phase 2] Grid は地形 + バンドル内座標 + 深度間隔から自動生成します。");
    println!("[Phase 3] u/v・水温・塩分は1つのバンドルから読み込みます。");
    println!("[Phase 4] ソースの深度からシミュレーション深度へ補間し、海底マスクを適用します。");
    println!("[Phase 5] 初期状態・境界条件・再開状態を生成します。");
    println!("[Phase 6] grid.snc / terrain.snc / initial.snc / boundary.snc / restart.snc / forcing.snc / manifest.json を出力します。");
    println!("[Phase 7] Simulator は設定ファイルの source_bundle から同じ生成経路を使用します。\n");

    if ask_yes_no("Grid・物理場を生成し、Simulator 用 input.sof を作成しますか？", true)? {
        let bundle = crate::pipeline::build_with_report(&config, &diagnostic)?;
        println!("\n生成完了: {}", config.output_root().join("input.sof").display());
        println!("格子: {} × {} × {} (lon × lat × depth)",
            bundle.field.velocity().grid().width(), bundle.field.velocity().grid().height(), bundle.depths.len());
        println!("地形: {} × {}", bundle.terrain.width, bundle.terrain.height);
    } else {
        println!("SOF 生成はスキップしました。後から設定済みの bundle を使って再実行できます。");
    }
    println!("\n完了。通常起動: cargo run -p simulator");
    Ok(())
}

fn ask_yes_no(question: &str, default: bool) -> Result<bool> {
    loop {
        let hint = if default { "[Y/n]" } else { "[y/N]" };
        print!("{} {} ", question, hint);
        io::stdout().flush()?;
        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 { return Ok(default); }
        match line.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Y または N で入力してください。"),
        }
    }
}

fn ask_text(question: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", question, default);
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 || line.trim().is_empty() {
        Ok(default.to_string())
    } else {
        Ok(line.trim().to_string())
    }
}

fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { root.join(path) }
}
