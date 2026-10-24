//! Source-data diagnostics shared by the CLI wizard and the normal dataset pipeline.
//!
//! The diagnostic is deliberately conservative: it detects structural and common
//! physical plausibility problems, but it does not certify scientific validity.
use anyhow::{Context, Result};
use serde::Serialize;
use std::{fs, path::PathBuf};
use s_sof::grid::{GridData, GridReader};

use crate::{config::Config, source_bundle::{SourceBundle, SourceCoordinates}};

#[derive(Clone, Debug, Serialize)]
pub struct DiagnosticReport {
    pub dataset: String,
    pub status: String,
    pub terrain: Option<TerrainSummary>,
    pub source: Option<SourceSummary>,
    pub files: Vec<FileSummary>,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<String>,
}

impl DiagnosticReport {
    pub fn has_errors(&self) -> bool { self.findings.iter().any(|f| f.severity == "ERROR") }
    pub fn has_warnings(&self) -> bool { self.findings.iter().any(|f| f.severity == "WARNING") }
    fn refresh_status(&mut self) {
        self.status = (if self.has_errors() { "FAIL" } else if self.has_warnings() { "WARN" } else { "PASS" }).to_string();
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TerrainSummary {
    pub width: usize, pub height: usize,
    pub longitude_min: f32, pub longitude_max: f32,
    pub latitude_min: f32, pub latitude_max: f32,
    pub elevation_min_m: f32, pub elevation_max_m: f32,
    pub wet_cells: usize, pub land_or_zero_cells: usize,
    pub estimated_max_depth_m: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceSummary {
    pub depth_levels: usize, pub latitude_points: usize, pub longitude_points: usize,
    pub depth_min_m: f32, pub depth_max_m: f32,
    pub latitude_min: f64, pub latitude_max: f64,
    pub longitude_min: f64, pub longitude_max: f64,
    pub has_velocity: bool, pub has_temperature: bool, pub has_salinity: bool,
    pub fields: Vec<FieldSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FieldSummary {
    pub name: String, pub samples: usize,
    pub min: f32, pub max: f32, pub mean: f64,
    pub zero_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileSummary { pub role: String, pub path: String, pub exists: bool, pub bytes: Option<u64> }

#[derive(Clone, Debug, Serialize)]
pub struct Finding { pub severity: String, pub code: String, pub message: String, pub recommendation: String }

pub fn diagnose(config: &Config) -> DiagnosticReport {
    let mut report = DiagnosticReport {
        dataset: config.name.clone(), status: "PASS".into(), terrain: None, source: None,
        files: Vec::new(), findings: Vec::new(), recommendations: Vec::new(),
    };

    let terrain_setting = config.input.terrain_source.as_ref().or(config.input.terrain_grid.as_ref());
    let terrain = if let Some(setting) = terrain_setting {
        let path = config.resolve(setting);
        inspect_file(&mut report, "terrain", &path);
        match GridReader::open(&path).and_then(|mut r| r.read()) {
            Ok(grid) => match inspect_terrain(&mut report, grid) {
                Some(t) => Some(t), None => None,
            },
            Err(e) => {
                finding(&mut report, "ERROR", "TERRAIN_READ", format!("地形データを読み込めません: {} ({e})", path.display()), "terrain_source のパスと SOF/SNC 形式を確認してください。");
                None
            }
        }
    } else {
        finding(&mut report, "ERROR", "TERRAIN_NOT_CONFIGURED", "input.terrain_source が設定されていません。", "地形SOF/SNCファイルを指定してください。");
        None
    };

    let source = load_source_for_diagnostics(config, &mut report);
    if let Some(source) = source {
        inspect_source(&mut report, source, terrain.as_ref());
    }

    if report.recommendations.is_empty() && !report.has_errors() && !report.has_warnings() {
        report.recommendations.push("元データの構造検査に問題は見つかりませんでした。科学的な妥当性は別途検証してください。".into());
    }
    report.refresh_status();
    report
}

pub fn print_report(report: &DiagnosticReport) {
    println!("\n============================================");
    println!(" SOS 元データ診断: {} [{}]", report.dataset, report.status);
    println!("============================================");
    for file in &report.files {
        let state = if file.exists { "OK" } else { "MISSING" };
        println!("[FILE:{state}] {} -> {}{}", file.role, file.path,
            file.bytes.map(|n| format!(" ({} bytes)", n)).unwrap_or_default());
    }
    if let Some(t) = &report.terrain {
        println!("[TERRAIN] {}×{}, lon {:.5}..{:.5}, lat {:.5}..{:.5}", t.width, t.height,
            t.longitude_min, t.longitude_max, t.latitude_min, t.latitude_max);
        println!("          elevation {:.1}..{:.1}m, wet cells {}, land/zero cells {}, max depth {:.1}m",
            t.elevation_min_m, t.elevation_max_m, t.wet_cells, t.land_or_zero_cells, t.estimated_max_depth_m);
    }
    if let Some(s) = &report.source {
        println!("[SOURCE] {} depth × {} lat × {} lon ({} samples/field)", s.depth_levels, s.latitude_points, s.longitude_points,
            s.depth_levels * s.latitude_points * s.longitude_points);
        println!("         depth {:.1}..{:.1}m, lon {:.5}..{:.5}, lat {:.5}..{:.5}", s.depth_min_m, s.depth_max_m,
            s.longitude_min, s.longitude_max, s.latitude_min, s.latitude_max);
        for f in &s.fields {
            println!("         {:<12} min={:>9.3} max={:>9.3} mean={:>9.3} zero={:>6.1}%", f.name, f.min, f.max, f.mean, f.zero_fraction * 100.0);
        }
        println!("         variables: velocity={}, temperature={}, salinity={}", s.has_velocity, s.has_temperature, s.has_salinity);
    }
    if report.findings.is_empty() { println!("[OK] 致命的な構造問題・既知の物理範囲警告はありません。"); }
    for f in &report.findings {
        println!("[{}:{}] {}", f.severity, f.code, f.message);
        println!("          対応: {}", f.recommendation);
    }
    for recommendation in &report.recommendations {
        println!("[NOTE] {}", recommendation);
    }
    println!("診断結果: {} (ERROR={}, WARNING={})\n", report.status,
        report.findings.iter().filter(|f| f.severity == "ERROR").count(),
        report.findings.iter().filter(|f| f.severity == "WARNING").count());
}

pub fn write_json(report: &DiagnosticReport, path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) { fs::create_dir_all(parent)?; }
    fs::write(path, serde_json::to_vec_pretty(report)?)
        .with_context(|| format!("診断JSONを書き込めません: {}", path.display()))
}

fn inspect_file(report: &mut DiagnosticReport, role: &str, path: &PathBuf) {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => report.files.push(FileSummary { role: role.into(), path: path.display().to_string(), exists: true, bytes: Some(meta.len()) }),
        Ok(_) => { report.files.push(FileSummary { role: role.into(), path: path.display().to_string(), exists: false, bytes: None }); finding(report, "ERROR", "NOT_A_FILE", format!("{} は通常ファイルではありません: {}", role, path.display()), "ファイルへのパスを指定してください。"); },
        Err(_) => { report.files.push(FileSummary { role: role.into(), path: path.display().to_string(), exists: false, bytes: None }); finding(report, "ERROR", "FILE_MISSING", format!("{} が見つかりません: {}", role, path.display()), "パスとファイル名を確認してください。"); },
    }
}

fn inspect_terrain(report: &mut DiagnosticReport, grid: GridData) -> Option<TerrainSummary> {
    let width = grid.longitude.len();
    let height = grid.latitude.len();
    if width < 2 || height < 2 || grid.bathymetry.len() != width.saturating_mul(height) {
        finding(report, "ERROR", "TERRAIN_SHAPE", format!("地形格子の形状が不正です: lon={}, lat={}, bathymetry={}", width, height, grid.bathymetry.len()), "longitude×latitude と bathymetry の要素数を一致させてください。");
        return None;
    }
    if grid.longitude.iter().chain(grid.latitude.iter()).chain(grid.bathymetry.iter()).any(|v| !v.is_finite()) {
        finding(report, "ERROR", "TERRAIN_NONFINITE", "地形または座標に NaN / Infinity が含まれています。", "地形ソースを再生成し、欠損値を正しく処理してください。");
        return None;
    }
    if grid.size[2] != height || grid.size[3] != width {
        finding(report, "WARNING", "TERRAIN_METADATA_SHAPE", format!("地形ヘッダーのサイズ [{}, {}, {}, {}] と座標配列 {}×{} が一致しません。", grid.size[0], grid.size[1], grid.size[2], grid.size[3], width, height), "地形ファイルを生成したときの grid_size と座標配列を確認してください。");
    }
    if grid.spacing.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        finding(report, "WARNING", "TERRAIN_SPACING", "地形ヘッダーに0以下または非有限の格子間隔があります。", "地形格子の spacing メタデータを確認してください。");
    }
    if !strictly_increasing_f32(&grid.longitude) || !strictly_increasing_f32(&grid.latitude) {
        finding(report, "ERROR", "TERRAIN_AXIS_ORDER", "地形の経度・緯度が昇順ではないか、重複しています。", "座標軸と bathymetry の並び順を合わせ、軸を昇順にしてください。");
    }
    if spacing_cv_f32(&grid.longitude) > 0.05 || spacing_cv_f32(&grid.latitude) > 0.05 {
        finding(report, "WARNING", "TERRAIN_NONUNIFORM_AXIS", "地形座標の間隔が大きく不均一です（変動係数 > 5%）。", "不規則格子を意図しているか確認してください。現在の描画/マスク処理は規則格子を前提とする箇所があります。");
    }
    let lon_min = min_f32(&grid.longitude); let lon_max = max_f32(&grid.longitude);
    let lat_min = min_f32(&grid.latitude); let lat_max = max_f32(&grid.latitude);
    let elev_min = min_f32(&grid.bathymetry); let elev_max = max_f32(&grid.bathymetry);
    let wet = grid.bathymetry.iter().filter(|v| **v < 0.0).count(); let land = grid.bathymetry.len() - wet;
    if wet == 0 { finding(report, "ERROR", "NO_WATER", "地形データに負の水深（海域）がありません。", "GEBCO等の標高符号を確認してください。海底は負値である必要があります。"); }
    if lon_min < -180.0 || lon_max > 360.0 || lat_min < -90.0 || lat_max > 90.0 {
        finding(report, "ERROR", "TERRAIN_COORD_RANGE", "地形の経度・緯度が地理座標の許容範囲外です。", "経度/緯度の順序と単位（度）を確認してください。");
    }
    if land == 0 { finding(report, "WARNING", "NO_LAND", "地形データ内に陸地/ゼロ標高セルがありません。", "対象領域が海域のみなら問題ありません。海岸を含む想定なら地形範囲を確認してください。"); }
    let summary = TerrainSummary { width, height, longitude_min: lon_min, longitude_max: lon_max,
        latitude_min: lat_min, latitude_max: lat_max, elevation_min_m: elev_min, elevation_max_m: elev_max,
        wet_cells: wet, land_or_zero_cells: land, estimated_max_depth_m: (-elev_min).max(0.0) };
    report.terrain = Some(summary.clone());
    Some(summary)
}

fn load_source_for_diagnostics(config: &Config, report: &mut DiagnosticReport) -> Option<SourceBundle> {
    if let Some(setting) = &config.input.source_bundle {
        let path = config.resolve(setting);
        inspect_file(report, "source_bundle", &path);
        return match SourceBundle::read(&path) {
            Ok(bundle) => Some(bundle),
            Err(e) => { finding(report, "ERROR", "SOURCE_BUNDLE_INVALID", format!("ソースバンドルを検証できません: {e:#}"), "バンドルを再生成してください: cargo run -p simulator -- generate"); None }
        };
    }
    let coord_path = match &config.input.source_coordinates {
        Some(p) => config.resolve(p),
        None => { finding(report, "ERROR", "SOURCE_NOT_CONFIGURED", "source_bundle / source_coordinates が設定されていません。", "source_bundle を指定するか、旧形式の座標ファイルを設定してください。"); return None; }
    };
    inspect_file(report, "source_coordinates", &coord_path);
    for (role, p) in [("velocity_u", &config.input.velocity_u), ("velocity_v", &config.input.velocity_v), ("temperature", &config.input.temperature), ("salinity", &config.input.salinity)] {
        if let Some(p) = p { inspect_file(report, role, &config.resolve(p)); }
    }
    let coordinates: SourceCoordinates = match fs::read(&coord_path)
        .with_context(|| format!("座標ファイルを読めません: {}", coord_path.display()))
        .and_then(|bytes| serde_json::from_slice::<SourceCoordinates>(&bytes).context("座標JSONの形式が不正です")) {
        Ok(value) => value,
        Err(e) => { finding(report, "ERROR", "LEGACY_COORDINATES_INVALID", format!("旧形式の座標を読めません: {e:#}"), "coordinates.json の JSON 構造と depth/latitude/longitude 配列を確認してください。"); return None; }
    };
    let expected = coordinates.depth.len().saturating_mul(coordinates.latitude.len()).saturating_mul(coordinates.longitude.len());
    let read_optional = |setting: &Option<PathBuf>, role: &str, report: &mut DiagnosticReport| -> Option<Vec<f32>> {
        let path = setting.as_ref().map(|p| config.resolve(p))?;
        match read_raw_f32(&path, expected) {
            Ok(values) => Some(values),
            Err(e) => { finding(report, "ERROR", "LEGACY_FIELD_INVALID", format!("{} を読み込めません: {e:#}", role), "f32 little-endian形式、配列要素数、shape=(depth, latitude, longitude) を確認してください。"); None }
        }
    };
    let u = read_optional(&config.input.velocity_u, "velocity_u", report);
    let v = read_optional(&config.input.velocity_v, "velocity_v", report);
    if u.is_some() != v.is_some() {
        finding(report, "ERROR", "VELOCITY_COMPONENT_PAIR", "旧形式の速度データは u と v をセットで指定してください。", "u/v 両方のパスを指定するか、両方を省略してください。");
    }
    let velocity_pair = u.is_some() && v.is_some();
    let temperature = read_optional(&config.input.temperature, "temperature", report);
    let salinity = read_optional(&config.input.salinity, "salinity", report);
    Some(SourceBundle { coordinates, velocity_u: if velocity_pair { u } else { None }, velocity_v: if velocity_pair { v } else { None }, temperature, salinity })
}

fn read_raw_f32(path: &std::path::Path, expected: usize) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("ファイルを読めません: {}", path.display()))?;
    anyhow::ensure!(bytes.len() % 4 == 0, "ファイルサイズ {} bytes は f32 の4バイト倍数ではありません", bytes.len());
    let values = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect::<Vec<_>>();
    anyhow::ensure!(values.len() == expected, "要素数 {} != 期待値 {}", values.len(), expected);
    anyhow::ensure!(values.iter().all(|v| v.is_finite()), "NaN / Infinity が含まれています");
    Ok(values)
}

fn inspect_source(report: &mut DiagnosticReport, source: SourceBundle, terrain: Option<&TerrainSummary>) {
    let c: &SourceCoordinates = &source.coordinates;
    let coords_ok = c.depth.len() >= 2 && c.latitude.len() >= 2 && c.longitude.len() >= 2
        && c.depth.iter().all(|v| v.is_finite()) && c.latitude.iter().all(|v| v.is_finite()) && c.longitude.iter().all(|v| v.is_finite())
        && c.depth.windows(2).all(|w| w[1] > w[0]) && c.latitude.windows(2).all(|w| w[1] > w[0]) && c.longitude.windows(2).all(|w| w[1] > w[0]);
    if !coords_ok { finding(report, "ERROR", "SOURCE_COORDINATES", "元データの座標軸が不足・非有限・非昇順です。", "depth/latitude/longitude をそれぞれ昇順かつ重複なしにしてください。"); }
    if c.latitude.len() >= 2 && spacing_cv_f64(&c.latitude) > 0.05 {
        finding(report, "WARNING", "SOURCE_LAT_NONUNIFORM", "元データの緯度格子間隔が不均一です（変動係数 > 5%）。", "GridBuilder は平均格子間隔を使うため、不規則格子なら再格子化してから使用してください。");
    }
    if c.longitude.len() >= 2 && spacing_cv_f64(&c.longitude) > 0.05 {
        finding(report, "WARNING", "SOURCE_LON_NONUNIFORM", "元データの経度格子間隔が不均一です（変動係数 > 5%）。", "GridBuilder は平均格子間隔を使うため、不規則格子なら再格子化してから使用してください。");
    }
    if c.depth.iter().any(|v| *v < 0.0) { finding(report, "WARNING", "NEGATIVE_DEPTH", "元データの深度に負値があります。", "深度が海面から下向きの正値か、標高の符号かを確認してください。"); }
    if c.latitude.iter().any(|v| *v < -90.0 || *v > 90.0) || c.longitude.iter().any(|v| *v < -180.0 || *v > 360.0) {
        finding(report, "ERROR", "SOURCE_COORD_RANGE", "元データの経度・緯度が地理座標の許容範囲外です。", "座標軸の入れ替わり、単位、経度表記を確認してください。");
    }
    let count = c.depth.len().saturating_mul(c.latitude.len()).saturating_mul(c.longitude.len());
    let mut fields = Vec::new();
    for (name, values, bounds) in [
        ("u", source.velocity_u.as_deref(), Some((-5.0, 5.0))),
        ("v", source.velocity_v.as_deref(), Some((-5.0, 5.0))),
        ("temperature_C", source.temperature.as_deref(), Some((-3.0, 40.0))),
        ("salinity_PSU", source.salinity.as_deref(), Some((0.0, 45.0))),
    ] {
        if let Some(values) = values {
            if values.len() != count {
                finding(report, "ERROR", "FIELD_LENGTH", format!("{} の要素数 {} が座標から期待される {} と一致しません。", name, values.len(), count), "元データの配列順と shape=(depth, latitude, longitude) を確認してください。");
                continue;
            }
            if values.is_empty() {
                finding(report, "ERROR", "FIELD_EMPTY", format!("{} が空です。", name), "座標軸と元データの配列要素数を確認してください。");
                continue;
            }
            if values.iter().any(|v| !v.is_finite()) {
                finding(report, "ERROR", "FIELD_NONFINITE", format!("{} に NaN / Infinity が含まれます。", name), "欠損値コードを有限値として残さず、欠損マスクまたは適切な補間を使用してください。");
                continue;
            }
            let min = min_f32(values); let max = max_f32(values);
            let mean = values.iter().map(|v| *v as f64).sum::<f64>() / values.len().max(1) as f64;
            let zero_fraction = values.iter().filter(|v| **v == 0.0).count() as f64 / values.len().max(1) as f64;
            if let Some((lo, hi)) = bounds {
                let suspicious = values.iter().filter(|v| **v < lo || **v > hi).count();
                if suspicious > 0 {
                    finding(report, "WARNING", "FIELD_RANGE", format!("{} に一般的な物理範囲外の値が {} 件あります (min={:.4}, max={:.4})。", name, suspicious, min, max), "単位、欠損値の符号化、変数の並び、データのスケールを確認してください。範囲は警告用の目安で、地域固有の品質判定ではありません。");
                }
            }
            if values.iter().all(|v| *v == values[0]) {
                finding(report, "WARNING", "FIELD_CONSTANT", format!("{} は全要素が同じ値 ({:.4}) です。", name, values[0]), "定数場が意図的か、入力ファイルの取り違え/読込失敗でないか確認してください。");
            }
            fields.push(FieldSummary { name: name.into(), samples: values.len(), min, max, mean, zero_fraction });
        }
    }
    if source.velocity_u.is_some() != source.velocity_v.is_some() {
        finding(report, "ERROR", "VELOCITY_COMPONENT_PAIR", "速度場は u と v の両成分が必要ですが、片方しかありません。", "u/v をセットで提供するか、両方を省略して明示的なゼロ初期化を選んでください。");
    }
    if source.velocity_u.is_none() { finding(report, "WARNING", "VELOCITY_MISSING", "速度データがありません。生成時に u=v=0 m/s として初期化されます。", "流線を実データで表示したい場合は u/v 元データを含めてください。"); }
    if source.temperature.is_none() { finding(report, "WARNING", "TEMPERATURE_MISSING", "水温データがありません。生成時に15°Cで初期化されます。", "科学的な計算では観測/再解析水温を入力してください。"); }
    if source.salinity.is_none() { finding(report, "WARNING", "SALINITY_MISSING", "塩分データがありません。生成時に35 PSUで初期化されます。", "科学的な計算では観測/再解析塩分を入力してください。"); }
    if let Some(t) = terrain {
        let source_lon_min = c.longitude.iter().copied().fold(f64::INFINITY, f64::min) as f32;
        let source_lon_max = c.longitude.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
        let source_lat_min = c.latitude.iter().copied().fold(f64::INFINITY, f64::min) as f32;
        let source_lat_max = c.latitude.iter().copied().fold(f64::NEG_INFINITY, f64::max) as f32;
        if source_lon_min < t.longitude_min || source_lon_max > t.longitude_max || source_lat_min < t.latitude_min || source_lat_max > t.latitude_max {
            finding(report, "WARNING", "DOMAIN_COVERAGE", "海洋ソースの領域が地形データの領域を一部超えています。最近傍地形セルでのマスクが端部で不正確になる可能性があります。", "地形を海洋データ領域より広く用意するか、領域を切り出して一致させてください。");
        }
        if c.depth.last().copied().unwrap_or(0.0) < t.estimated_max_depth_m * 0.5 {
            finding(report, "WARNING", "DEPTH_COVERAGE", format!("元データの最大深度 {:.1}m に対して、地形の最大水深は {:.1}m です。深部の多くは外挿されます。", c.depth.last().copied().unwrap_or(0.0), t.estimated_max_depth_m), "より深い海洋データを取得するか、外挿を意図した初期化であることを記録してください。");
        }
    }
    if c.depth.len() >= 2 && c.depth[0] > 0.0 {
        finding(report, "WARNING", "SURFACE_NOT_INCLUDED", format!("最浅の元データ深度は {:.1}m で、海面深度0mがありません。", c.depth[0]), "海面まで補間する際の外挿が意図どおりか確認してください。");
    }
    report.source = Some(SourceSummary {
        depth_levels: c.depth.len(), latitude_points: c.latitude.len(), longitude_points: c.longitude.len(),
        depth_min_m: c.depth.first().copied().unwrap_or(f32::NAN), depth_max_m: c.depth.last().copied().unwrap_or(f32::NAN),
        latitude_min: c.latitude.first().copied().unwrap_or(f64::NAN), latitude_max: c.latitude.last().copied().unwrap_or(f64::NAN),
        longitude_min: c.longitude.first().copied().unwrap_or(f64::NAN), longitude_max: c.longitude.last().copied().unwrap_or(f64::NAN),
        has_velocity: source.velocity_u.is_some() && source.velocity_v.is_some(), has_temperature: source.temperature.is_some(), has_salinity: source.salinity.is_some(), fields,
    });
    report.recommendations.push("この診断は構造・一般的な値域の事前確認です。元データの出典、時刻、単位、格子定義、観測品質はメタデータと照合してください。".into());
}

fn finding(report: &mut DiagnosticReport, severity: &str, code: &str, message: impl Into<String>, recommendation: impl Into<String>) {
    report.findings.push(Finding { severity: severity.into(), code: code.into(), message: message.into(), recommendation: recommendation.into() });
}
fn min_f32(values: &[f32]) -> f32 { values.iter().copied().fold(f32::INFINITY, f32::min) }
fn max_f32(values: &[f32]) -> f32 { values.iter().copied().fold(f32::NEG_INFINITY, f32::max) }

fn strictly_increasing_f32(values: &[f32]) -> bool { values.windows(2).all(|w| w[1] > w[0]) }
fn spacing_cv_f32(values: &[f32]) -> f64 {
    let steps = values.windows(2).map(|w| (w[1] - w[0]).abs() as f64).collect::<Vec<_>>();
    spacing_cv(&steps)
}
fn spacing_cv_f64(values: &[f64]) -> f64 {
    let steps = values.windows(2).map(|w| (w[1] - w[0]).abs()).collect::<Vec<_>>();
    spacing_cv(&steps)
}
fn spacing_cv(steps: &[f64]) -> f64 {
    if steps.len() < 2 { return 0.0; }
    let mean = steps.iter().sum::<f64>() / steps.len() as f64;
    if mean <= 0.0 { return f64::INFINITY; }
    let variance = steps.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / steps.len() as f64;
    variance.sqrt() / mean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_severity_gate_works() {
        let mut report = DiagnosticReport { dataset: "test".into(), status: "PASS".into(), terrain: None, source: None, files: vec![], findings: vec![], recommendations: vec![] };
        assert!(!report.has_errors());
        assert!(!report.has_warnings());
        finding(&mut report, "WARNING", "TEST", "warning", "fix");
        report.refresh_status();
        assert_eq!(report.status, "WARN");
        finding(&mut report, "ERROR", "TEST_ERROR", "error", "fix");
        report.refresh_status();
        assert_eq!(report.status, "FAIL");
        assert!(report.has_errors());
    }

    #[test]
    fn regular_axes_have_zero_spacing_variation() {
        assert!(spacing_cv_f64(&[1.0, 2.0, 3.0, 4.0]) < 1.0e-12);
        assert!(spacing_cv_f64(&[1.0, 2.0, 4.0]) > 0.0);
    }
}
