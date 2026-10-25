//! Native Rust input pipeline: public NetCDF / geographic GeoTIFF -> normalized arrays -> s-sof.
//! No external interpreter or preprocessing scripts are used.
use anyhow::{Context, Result, ensure, anyhow};
use serde::{Deserialize, Serialize};
use s_field::Grid4D;
use s_sof::grid::GridWriter;
use std::{fs, path::Path, process::{Command, Stdio}, time::UNIX_EPOCH, hash::Hasher};
use crate::{config::{Config, NetcdfInputConfig}, source_bundle::SourceBundle};

#[derive(Debug, Serialize, Deserialize)]
struct TerrainJson { longitude: Vec<f64>, latitude: Vec<f64>, elevation_m: Vec<f32>, shape: Vec<usize> }

pub fn is_configured(config: &Config) -> bool { config.input.netcdf.is_some() }

pub fn prepare(config: &Config, _config_path: &Path) -> Result<()> {
    let Some(nc) = config.input.netcdf.as_ref() else { return Ok(()); };
    let terrain_source = config.input.terrain_source.as_ref().or(nc.terrain_file.as_ref())
        .ok_or_else(|| anyhow!("input.terrain_source に公開地形データ(.nc/.tif/.tiff)を指定してください"))?;
    let terrain_path = config.resolve(terrain_source);
    let ocean_path = config.resolve(config.hycom_output_path(nc));
    ensure!(terrain_path.is_file(), "地形データがありません: {}。NetCDFまたはEPSG:4326 GeoTIFFを配置してください。", terrain_path.display());
    ensure!(ocean_path.is_file(), "海洋NetCDFがありません: {}。先に `cargo run -p simulator -- get --hycom` を実行してください。", ocean_path.display());
    let prepared = config.resolve(&nc.prepared_dir);
    let raw_dir = prepared.join("raw_arrays");
    fs::create_dir_all(&raw_dir)?;
    let manifest_path = prepared.join("preprocess_manifest.json");
    let terrain_json_path = prepared.join("terrain.json");
    let signature = cache_signature(nc, &config.study.goal, &terrain_path, &ocean_path)?;
    let mut required = vec!["coordinates.json"];
    for field in config.study.requested_fields()? {
        required.push(match field { "u" => "u.bin", "v" => "v.bin", "w" => "w.bin", "temperature" => "temperature.bin", "salinity" => "salinity.bin", _ => continue });
    }
    let cached = manifest_path.is_file() && terrain_json_path.is_file() && required.iter().all(|n| raw_dir.join(n).is_file())
        && fs::read_to_string(&manifest_path).unwrap_or_default().contains(&format!("\"sos_cache_signature\": \"{signature}\""));
    if !cached {
        println!("[Rust NetCDF] 公開入力を前処理します: {} + {}", terrain_path.display(), ocean_path.display());
        let mut terrain = if is_geotiff(&terrain_path) { read_geotiff(&terrain_path, nc)? } else { read_terrain_netcdf(&terrain_path, nc)? };
        validate_terrain(&terrain)?;
        // Use the actual HYCOM coordinate centres as the common horizontal domain.
        // The requested rectangle is only a query window; both datasets are sampled
        // on their own grids, so their pixel/point centres do not exactly equal its edges.
        let ocean_bounds = selected_ocean_bounds(&ocean_path, nc)?;
        terrain = clip_terrain_to_bounds(terrain, ocean_bounds)?;
        validate_terrain(&terrain)?;
        println!("[Grid alignment] requested region: lon {:.5}..{:.5}, lat {:.5}..{:.5}",
            nc.region.longitude_min, nc.region.longitude_max, nc.region.latitude_min, nc.region.latitude_max);
        println!("[Grid alignment] HYCOM point domain: lon {:.5}..{:.5}, lat {:.5}..{:.5}",
            ocean_bounds.0, ocean_bounds.1, ocean_bounds.2, ocean_bounds.3);
        println!("[Grid alignment] terrain render domain clipped to HYCOM coverage: lon {:.5}..{:.5}, lat {:.5}..{:.5}",
            terrain.longitude[0], *terrain.longitude.last().unwrap(), terrain.latitude[0], *terrain.latitude.last().unwrap());
        fs::write(&terrain_json_path, serde_json::to_vec(&terrain)?)?;
        read_ocean_netcdf(&ocean_path, nc, &raw_dir, &manifest_path, &terrain_path, &signature, &config.study.requested_fields()?)?;
    } else {
        println!("[Rust NetCDF] 前処理キャッシュを再利用します: {}", prepared.display());
    }

    let terrain: TerrainJson = serde_json::from_slice(&fs::read(&terrain_json_path)?)?;
    let h = terrain.shape[0]; let w = terrain.shape[1];
    let lon: Vec<f32> = terrain.longitude.iter().map(|v| *v as f32).collect();
    let lat: Vec<f32> = terrain.latitude.iter().map(|v| *v as f32).collect();
    let mean_lat = lat.iter().map(|v| *v as f64).sum::<f64>() / h as f64;
    let dx = average_step(&lon) * 111_320.0 * mean_lat.to_radians().cos().abs() as f32;
    let dy = average_step(&lat) * 110_540.0;
    let grid = Grid4D::new([1, 1, h, w], [1.0, 1.0, dy.max(1.0), dx.max(1.0)]);
    let terrain_target = config.input.terrain_output.as_ref().ok_or_else(|| anyhow!("input.terrain_output を設定してください"))?;
    let terrain_snc = config.resolve(terrain_target);
    if let Some(p) = terrain_snc.parent() { fs::create_dir_all(p)?; }
    let mut writer = GridWriter::create(&terrain_snc).context("s-sof GridWriterを作成できません")?;
    writer.write(&grid, &lat, &lon, &[0.0], &terrain.elevation_m).context("terrain SNCへの書き込みに失敗しました")?;
    writer.finish()?;
    println!("[s-sof] 地形 → {} ({}×{})", terrain_snc.display(), w, h);
    let bundle = SourceBundle::from_legacy_dir(
        &raw_dir, raw_dir.join("u.bin").is_file() && raw_dir.join("v.bin").is_file(),
        raw_dir.join("temperature.bin").is_file(), raw_dir.join("salinity.bin").is_file(),
    )?;
    let bundle_target = config.input.source_bundle.as_ref().ok_or_else(|| anyhow!("input.source_bundle が必要です"))?;
    let bundle_path = config.resolve(bundle_target);
    bundle.write(&bundle_path)?;
    println!("[Rust NetCDF] 物理場 → {}", bundle_path.display());
    Ok(())
}

pub fn download_hycom(config: &Config, force: bool) -> Result<()> {
    let nc = config.input.netcdf.as_ref().ok_or_else(|| anyhow!("HYCOM取得には input.netcdf 設定が必要です"))?;
    let output = config.resolve(config.hycom_output_path(nc));
    if output.is_file() && fs::metadata(&output)?.len() > 1024 && !force {
        println!("[HYCOM] 既存データを再利用: {} (再取得は --force)", output.display());
        report_hycom_domain(&output, nc)?;
        return Ok(());
    }

    // NCSS requests must target /thredds/ncss/grid/... . The older URL used
    // /thredds/ncss/... and `time=last`, which NCSS rejects with HTTP 400.
    let configured_base = nc.hycom_url.as_deref()
        .unwrap_or("https://ncss.hycom.org/thredds/ncss/grid/GLBy0.08/expt_93.0");
    let mut base = configured_base
        .replace("/thredds/dodsC/", "/thredds/ncss/grid/")
        .replace("/dodsC/", "/ncss/grid/");
    if base.contains("/thredds/ncss/") && !base.contains("/thredds/ncss/grid/") {
        base = base.replacen("/thredds/ncss/", "/thredds/ncss/grid/", 1);
    }
    if base.contains("/thredds/ncss/grid/") {
        // no-op: this is the canonical NCSS Grid endpoint
    } else if !base.contains("/thredds/ncss/") {
        return Err(anyhow!("input.netcdf.hycom_url は NCSS endpoint を指定してください（例: https://ncss.hycom.org/thredds/ncss/grid/GLBy0.08/expt_93.0）"));
    }

    let r = &nc.region;
    let requested = config.study.requested_fields()?;
    let mut variables = Vec::new();
    for field in &requested {
        // GLBy0.08/expt_93.0 does not publish water_w. The Rust preprocessing
        // stage derives w from u/v using incompressible continuity instead.
        if *field == "w" { continue; }
        let default_name = match *field {
            "u" => "water_u", "v" => "water_v",
            "temperature" => "water_temp", "salinity" => "salinity", _ => continue,
        };
        let variable = nc.variables.get(*field).map(String::as_str).unwrap_or(default_name);
        if !variables.iter().any(|existing: &String| existing == variable) {
            variables.push(variable.to_string());
        }
    }
    ensure!(!variables.is_empty(), "study.goal に対応するHYCOM変数がありません");

    // GLBy0.08/expt_93.0 is an archive, not a live feed. Its published archive
    // coverage ends on 2024-09-04 (t023 = 21:00 UTC). NCSS expects an ISO timestamp;
    // the pseudo-value `last` is not valid for its `time` parameter.
    let time = nc.hycom_time.as_deref().unwrap_or("2024-09-04T21:00:00Z");
    let encoded_time = time.replace(':', "%3A");
    let query = format!(
        "var={}&north={}&west={}&east={}&south={}&time={}&horizStride=1&vertStride=1&accept=netcdf4",
        variables.join(","), r.latitude_max, r.longitude_min, r.longitude_max,
        r.latitude_min, encoded_time
    );
    let url = format!("{}?{}", base.trim_end_matches('?').trim_end_matches('/'), query);
    if let Some(p) = output.parent() { fs::create_dir_all(p)?; }
    let tmp = output.with_extension("nc.download.tmp");
    println!("[HYCOM] 公開NCSSから対象領域を取得します: {}", url);
    println!("[HYCOM] 要求時刻: {}（expt_93.0 は2024年9月までのアーカイブ）", time);

    // Keep the HTTP response body on disk even for HTTP 400/500 so the error
    // message can show the server's diagnostic instead of only a generic curl error.
    let status_file = output.with_extension("http_status.tmp");
    let status_handle = fs::File::create(&status_file)?;
    let result = Command::new("curl")
        .args([
            "--location", "--show-error", "--progress-bar",
            "--connect-timeout", "20", "--max-time", "1800",
            "--retry", "2", "--retry-delay", "2", "--output",
        ])
        .arg(&tmp)
        .arg("--write-out")
        .arg("%{http_code}")
        .arg(&url)
        .stdout(Stdio::from(status_handle))
        .status()
        .context("curlを起動できません。macOS標準環境またはHomebrewのcurlを確認してください")?;
    if !result.success() {
        let _ = fs::remove_file(&tmp);
        let _ = fs::remove_file(&status_file);
        return Err(anyhow!("HYCOMへのダウンロードが失敗しました（curl終了コード: {:?}）。接続・転送のタイムアウトはそれぞれ20秒・30分です。", result.code()));
    }
    let status_text = fs::read_to_string(&status_file).unwrap_or_default().trim().to_string();
    let _ = fs::remove_file(&status_file);
    if status_text != "200" {
        let body = fs::read(&tmp).map(|bytes| String::from_utf8_lossy(&bytes).chars().take(700).collect::<String>()).unwrap_or_default();
        let _ = fs::remove_file(&tmp);
        return Err(anyhow!("HYCOM NCSS が HTTP {} を返しました。要求時刻・変数名・endpointを確認してください。サーバー応答: {}", status_text, body.replace('\n', " ")));
    }
    ensure!(fs::metadata(&tmp)?.len() > 1024, "HYCOMサーバーの応答がNetCDFとして小さすぎます");
    netcdf::open(&tmp).context("HYCOM取得結果をNetCDFとして開けません。NCSS endpoint/queryを確認してください")?;
    report_hycom_domain(&tmp, nc)?;
    fs::rename(&tmp, &output).with_context(|| format!("取得ファイルを保存できません: {}", output.display()))?;
    println!("[HYCOM] 保存完了: {} ({} bytes)", output.display(), fs::metadata(&output)?.len());
    Ok(())
}

/// Print and validate the coordinate coverage actually present in a downloaded HYCOM file.
fn report_hycom_domain(path: &Path, nc: &NetcdfInputConfig) -> Result<()> {
    let file = netcdf::open(path).with_context(|| format!("HYCOM NetCDFを開けません: {}", path.display()))?;
    let (_, _, lon) = axis(&file, nc, "longitude")?;
    let (_, _, lat) = axis(&file, nc, "latitude")?;
    let r = &nc.region;
    let xi = crop_axis(&lon, r.longitude_min, r.longitude_max, false);
    let yi = crop_axis(&lat, r.latitude_min, r.latitude_max, false);
    ensure!(xi.len() >= 2 && yi.len() >= 2,
        "HYCOMの実際の座標範囲が設定領域と合いません。requested lon {:.5}..{:.5}, lat {:.5}..{:.5}; selected points lon={}, lat={}",
        r.longitude_min, r.longitude_max, r.latitude_min, r.latitude_max, xi.len(), yi.len());
    let lon_min = xi.iter().map(|i| lon[*i]).fold(f64::INFINITY, f64::min);
    let lon_max = xi.iter().map(|i| lon[*i]).fold(f64::NEG_INFINITY, f64::max);
    let lat_min = yi.iter().map(|i| lat[*i]).fold(f64::INFINITY, f64::min);
    let lat_max = yi.iter().map(|i| lat[*i]).fold(f64::NEG_INFINITY, f64::max);
    println!("[HYCOM domain] requested lon {:.5}..{:.5}, lat {:.5}..{:.5}", r.longitude_min, r.longitude_max, r.latitude_min, r.latitude_max);
    println!("[HYCOM domain] actual grid lon {:.5}..{:.5} ({} points), lat {:.5}..{:.5} ({} points)", lon_min, lon_max, xi.len(), lat_min, lat_max, yi.len());
    Ok(())
}

/// Return the actual selected HYCOM coordinate-centre bounds (not the request rectangle).
fn selected_ocean_bounds(path: &Path, nc: &NetcdfInputConfig) -> Result<(f64, f64, f64, f64)> {
    let file = netcdf::open(path).with_context(|| format!("HYCOM NetCDFを開けません: {}", path.display()))?;
    let (_, _, lon) = axis(&file, nc, "longitude")?;
    let (_, _, lat) = axis(&file, nc, "latitude")?;
    let r = &nc.region;
    let xi = crop_axis(&lon, r.longitude_min, r.longitude_max, false);
    let yi = crop_axis(&lat, r.latitude_min, r.latitude_max, false);
    ensure!(xi.len() >= 2 && yi.len() >= 2, "HYCOMと共通化できる領域がありません: lon points={}, lat points={}", xi.len(), yi.len());
    Ok((xi.iter().map(|i| lon[*i]).fold(f64::INFINITY, f64::min),
        xi.iter().map(|i| lon[*i]).fold(f64::NEG_INFINITY, f64::max),
        yi.iter().map(|i| lat[*i]).fold(f64::INFINITY, f64::min),
        yi.iter().map(|i| lat[*i]).fold(f64::NEG_INFINITY, f64::max)))
}

/// Clip the high-resolution terrain to the selected HYCOM coordinate-centre extent,
/// so rendered land/sea and the solver's horizontal grid describe the same domain.
fn clip_terrain_to_bounds(t: TerrainJson, bounds: (f64, f64, f64, f64)) -> Result<TerrainJson> {
    let (lon_min, lon_max, lat_min, lat_max) = bounds;
    let xs: Vec<usize> = t.longitude.iter().enumerate().filter_map(|(i, x)| (*x >= lon_min && *x <= lon_max).then_some(i)).collect();
    let ys: Vec<usize> = t.latitude.iter().enumerate().filter_map(|(i, y)| (*y >= lat_min && *y <= lat_max).then_some(i)).collect();
    ensure!(xs.len() >= 2 && ys.len() >= 2,
        "地形とHYCOMの座標範囲が一致しません。HYCOM lon {:.5}..{:.5}, lat {:.5}..{:.5}; terrain selected {}×{}",
        lon_min, lon_max, lat_min, lat_max, xs.len(), ys.len());
    let mut elevation = Vec::with_capacity(xs.len() * ys.len());
    for &y in &ys { for &x in &xs { elevation.push(t.elevation_m[y * t.shape[1] + x]); } }
    Ok(TerrainJson {
        longitude: xs.iter().map(|i| t.longitude[*i]).collect(),
        latitude: ys.iter().map(|i| t.latitude[*i]).collect(),
        elevation_m: elevation,
        shape: vec![ys.len(), xs.len()],
    })
}

fn is_geotiff(path: &Path) -> bool { matches!(path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str(), "tif" | "tiff") }
fn aliases(role: &str) -> &'static [&'static str] { match role {
    "longitude" => &["longitude", "lon", "LONGITUDE", "x"], "latitude" => &["latitude", "lat", "LATITUDE", "y"],
    "depth" => &["depth", "deptht", "lev", "level", "z", "DEPTH"], "time" => &["time", "MT", "datetime", "ocean_time"],
    "terrain" => &["elevation", "z", "gebco", "Band1", "altitude", "topography"],
    "u" => &["water_u", "u_velocity", "uo", "u", "eastward_sea_water_velocity"], "v" => &["water_v", "v_velocity", "vo", "v", "northward_sea_water_velocity"],
    "w" => &["water_w", "wo", "w", "upward_sea_water_velocity"], "temperature" => &["water_temp", "thetao", "temperature", "temp", "sea_water_potential_temperature"],
    "salinity" => &["salinity", "water_salinity", "so", "salt", "sea_water_salinity"], _ => &[] } }
fn var_name(file: &netcdf::File, nc: &NetcdfInputConfig, key: &str, required: bool) -> Result<Option<String>> {
    // Own the fallback list so the catch-all branch never returns a slice
    // borrowing the short-lived match binding (`other`).
    let config_keys: Vec<&str> = match key {
        "longitude" => vec!["longitude", "ocean_longitude", "terrain_longitude"],
        "latitude" => vec!["latitude", "ocean_latitude", "terrain_latitude"],
        "terrain" => vec!["terrain", "terrain_elevation"],
        other => vec![other],
    };
    for config_key in config_keys {
        if let Some(v) = nc.variables.get(config_key) {
            if file.variable(v).is_some() { return Ok(Some(v.clone())); }
        }
    }
    for name in aliases(key) { if file.variable(name).is_some() { return Ok(Some((*name).to_string())); } }
    if required { anyhow::bail!("NetCDFで {key} を検出できません。input.netcdf.variables で変数名を指定してください"); }
    Ok(None)
}
fn axis(file: &netcdf::File, nc: &NetcdfInputConfig, key: &str) -> Result<(String, String, Vec<f64>)> {
    let name = var_name(file, nc, key, true)?.unwrap();
    let var = file.variable(&name).unwrap(); let dims = var.dimensions();
    ensure!(dims.len() == 1, "座標 {name} は1次元である必要があります");
    let values = var.get_values::<f64, _>(..).with_context(|| format!("座標 {name} を読み込めません"))?;
    ensure!(values.len() >= 2 && values.iter().all(|v| v.is_finite()), "座標 {name} の値が不正です");
    Ok((name, dims[0].name().to_string(), values))
}
fn crop_axis(values: &[f64], min: f64, max: f64, abs_depth: bool) -> Vec<usize> {
    let mut out: Vec<usize> = values.iter().enumerate().filter_map(|(i, v)| { let n = if abs_depth { v.abs() } else { *v }; (n >= min && n <= max).then_some(i) }).collect();
    out.sort_by(|a,b| { let va = if abs_depth { values[*a].abs() } else { values[*a] }; let vb = if abs_depth { values[*b].abs() } else { values[*b] }; va.total_cmp(&vb) }); out
}
fn read_terrain_netcdf(path: &Path, nc: &NetcdfInputConfig) -> Result<TerrainJson> {
    let file = netcdf::open(path).with_context(|| format!("NetCDF地形を開けません: {}", path.display()))?;
    let (_, xdim, xall) = axis(&file, nc, "longitude")?; let (_, ydim, yall) = axis(&file, nc, "latitude")?;
    let r = &nc.region; let xi = crop_axis(&xall, r.longitude_min, r.longitude_max, false); let yi = crop_axis(&yall, r.latitude_min, r.latitude_max, false);
    ensure!(xi.len() >= 2 && yi.len() >= 2, "NetCDF地形の対象範囲は2×2以上の格子が必要です");
    let x: Vec<f64> = xi.iter().map(|i| xall[*i]).collect(); let y: Vec<f64> = yi.iter().map(|i| yall[*i]).collect();
    let key = var_name(&file, nc, "terrain", true)?.unwrap(); let var = file.variable(&key).unwrap();
    let dims = var.dimensions(); let shape: Vec<usize> = dims.iter().map(|d| d.len()).collect(); let names: Vec<String> = dims.iter().map(|d| d.name().to_string()).collect();
    let values = var.get_values::<f64, _>(..).with_context(|| format!("地形変数 {key} を読み込めません"))?;
    ensure!(values.len() == shape.iter().product::<usize>(), "地形変数の配列サイズが不正です");
    let mut elevation = Vec::with_capacity(x.len()*y.len());
    for &iy in &yi { for &ix in &xi {
        let mut index = vec![0usize; shape.len()];
        for (d, name) in names.iter().enumerate() { if name == &xdim { index[d] = ix; } else if name == &ydim { index[d] = iy; } else { ensure!(shape[d] == 1, "地形変数 {key} に未対応の次元 {name} があります"); } }
        let flat = flatten(&index, &shape); let v = values[flat]; ensure!(v.is_finite() && v.abs() < 1.0e20, "地形変数に欠損値があります"); elevation.push(v as f32);
    }}
    Ok(TerrainJson { longitude: x, latitude: y, elevation_m: elevation, shape: vec![yi.len(), xi.len()] })
}
fn read_geotiff(path: &Path, nc: &NetcdfInputConfig) -> Result<TerrainJson> {
    // Use GDAL command-line tools instead of the gdal Rust crate. This avoids
    // generated-binding incompatibilities between gdal-sys and newer system GDAL.
    // gdalwarp handles projected CRS, geographic CRS, and rotated source rasters.
    let r = &nc.region;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("terrain");
    let base = std::env::temp_dir().join(format!("sos_reproject_{}_{}", std::process::id(), stem));
    let output = base.with_extension("tif");
    let xyz = base.with_extension("xyz");
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(&xyz);

    let status = Command::new("gdalwarp")
        .arg("-overwrite")
        .args(["-t_srs", "EPSG:4326", "-te_srs", "EPSG:4326", "-te"])
        .arg(r.longitude_min.to_string())
        .arg(r.latitude_min.to_string())
        .arg(r.longitude_max.to_string())
        .arg(r.latitude_max.to_string())
        .args(["-r", "bilinear", "-dstnodata", "-9999"])
        .arg(path)
        .arg(&output)
        .status()
        .with_context(|| "GeoTIFFの読み込み・再投影にはGDALの gdalwarp が必要です。macOSでは `brew install gdal` を実行してください")?;
    ensure!(status.success() && output.is_file(), "gdalwarp によるEPSG:4326への再投影に失敗しました: {}", path.display());

    let status = Command::new("gdal_translate")
        .args(["-q", "-of", "XYZ"])
        .arg(&output)
        .arg(&xyz)
        .status()
        .with_context(|| "GeoTIFFのラスタ値読み込みにはGDALの gdal_translate が必要です")?;
    ensure!(status.success() && xyz.is_file(), "gdal_translate によるGeoTIFFのXYZ変換に失敗しました: {}", path.display());

    let parsed = parse_xyz_terrain(&xyz);
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(&xyz);
    parsed.with_context(|| format!("GeoTIFFを読み込めません: {}", path.display()))
}

/// Read GDAL XYZ output (pixel-center x y z triples) into a regular WGS84 grid.
/// Rows in XYZ are usually north-to-south; output terrain arrays are south-to-north
/// to match the simulator's increasing latitude convention.
fn parse_xyz_terrain(path: &Path) -> Result<TerrainJson> {
    let text = fs::read_to_string(path).with_context(|| format!("XYZファイルを読み込めません: {}", path.display()))?;
    let mut triples: Vec<(f64, f64, f64)> = Vec::new();
    let mut longitude: Vec<f64> = Vec::new();
    let mut latitude: Vec<f64> = Vec::new();
    for (line_no, line) in text.lines().enumerate() {
        let mut parts = line.split_whitespace();
        let (Some(xs), Some(ys), Some(zs)) = (parts.next(), parts.next(), parts.next()) else { continue; };
        let x: f64 = xs.parse().with_context(|| format!("XYZ {}行目の経度が不正です", line_no + 1))?;
        let y: f64 = ys.parse().with_context(|| format!("XYZ {}行目の緯度が不正です", line_no + 1))?;
        let z: f64 = zs.parse().with_context(|| format!("XYZ {}行目の標高が不正です", line_no + 1))?;
        ensure!(x.is_finite() && y.is_finite(), "XYZに不正な座標があります ({}行目)", line_no + 1);
        longitude.push(x);
        latitude.push(y);
        triples.push((x, y, z));
    }
    ensure!(!triples.is_empty(), "GeoTIFFから画素値を取得できませんでした");
    longitude.sort_by(f64::total_cmp);
    latitude.sort_by(f64::total_cmp);
    longitude.dedup_by(|a, b| (*a - *b).abs() < 1.0e-10);
    latitude.dedup_by(|a, b| (*a - *b).abs() < 1.0e-10);
    ensure!(longitude.len() >= 2 && latitude.len() >= 2, "GeoTIFFの対象海域は2×2以上の格子が必要です");
    ensure!(triples.len() == longitude.len() * latitude.len(), "GeoTIFFのXYZ出力が規則格子ではありません");

    let width = longitude.len();
    let height = latitude.len();
    let mut elevation = vec![f32::NAN; width * height];
    for (x, y, z) in triples {
        let ix = nearest_axis_index(&longitude, x)?;
        let iy = nearest_axis_index(&latitude, y)?;
        ensure!(z.is_finite() && z.abs() < 1.0e20 && (z + 9999.0).abs() > 1.0e-6,
            "GeoTIFFの対象海域にNoData/欠損値があります。地形データの範囲またはNoData設定を確認してください");
        elevation[iy * width + ix] = z as f32;
    }
    ensure!(elevation.iter().all(|v| v.is_finite()), "GeoTIFFの再投影格子に未設定の画素があります");
    Ok(TerrainJson { longitude, latitude, elevation_m: elevation, shape: vec![height, width] })
}

fn nearest_axis_index(axis: &[f64], value: f64) -> Result<usize> {
    let idx = axis.binary_search_by(|v| v.total_cmp(&value)).unwrap_or_else(|i| i.min(axis.len().saturating_sub(1)));
    let mut best = idx;
    if idx > 0 && (axis[idx - 1] - value).abs() < (axis[best] - value).abs() { best = idx - 1; }
    ensure!((axis[best] - value).abs() < 1.0e-8, "XYZ座標を格子に割り当てられません");
    Ok(best)
}

fn read_ocean_netcdf(path: &Path, nc: &NetcdfInputConfig, raw: &Path, manifest: &Path, terrain: &Path, signature: &str, requested_fields: &[&str]) -> Result<()> {
    let file = netcdf::open(path).with_context(|| format!("海洋NetCDFを開けません: {}", path.display()))?;
    let (_, xdim, xall)=axis(&file,nc,"longitude")?; let (_, ydim, yall)=axis(&file,nc,"latitude")?; let (_, zdim,zall)=axis(&file,nc,"depth")?;
    let r=&nc.region; let xi=crop_axis(&xall,r.longitude_min,r.longitude_max,false); let yi=crop_axis(&yall,r.latitude_min,r.latitude_max,false); let zi=crop_axis(&zall,0.0,f64::INFINITY,true);
    ensure!(xi.len()>=2 && yi.len()>=2 && zi.len()>=2,"海洋NetCDFの対象領域に2×2×2以上の格子がありません");
    let depth:Vec<f32>=zi.iter().map(|i|zall[*i].abs() as f32).collect();
    ensure!(depth.windows(2).all(|w|w[1]>w[0]),"深度座標が昇順または一意ではありません");
    println!("[NetCDF] 深度方向: 全{}層を保持（{:.1}～{:.1} m）", depth.len(), depth[0], *depth.last().unwrap());
    let longitude:Vec<f64>=xi.iter().map(|i|xall[*i]).collect(); let latitude:Vec<f64>=yi.iter().map(|i|yall[*i]).collect();
    // Remove fields not requested by the current goal to prevent stale data from being bundled.
    for key in ["u", "v", "w", "temperature", "salinity"] { let _ = fs::remove_file(raw.join(format!("{key}.bin"))); }
    let time_name=var_name(&file,nc,"time",false)?;
    let time_dim=time_name.as_ref().and_then(|n|file.variable(n)).and_then(|v|v.dimensions().first().map(|d|d.name().to_string()));
    for key in ["u","v","w","temperature","salinity"] {
        if !requested_fields.contains(&key) { continue; }
        // Prefer a source-provided vertical velocity if the selected NetCDF has
        // one. For the configured HYCOM expt_93.0 archive it is absent, so
        // estimate w from u/v and the continuity equation below.
        let source_name = var_name(&file,nc,key,key != "w")?;
        if key == "w" && source_name.is_none() {
            let u = read_f32_array(&raw.join("u.bin"))?;
            let v = read_f32_array(&raw.join("v.bin"))?;
            let estimated = estimate_vertical_velocity(&u, &v, &depth, &latitude, &longitude)?;
            write_f32(&raw.join("w.bin"), &estimated)?;
            println!("[NetCDF] w: u/v から連続の式で推定 ({} values; source-observedではありません)", estimated.len());
            continue;
        }
        let Some(name) = source_name else { continue; };
        let var=file.variable(&name).unwrap(); let dims=var.dimensions(); let shape:Vec<usize>=dims.iter().map(|d|d.len()).collect(); let names:Vec<String>=dims.iter().map(|d|d.name()).collect();
        let values=var.get_values::<f64,_>(..).with_context(||format!("物理変数 {name} を読み込めません"))?;
        let mut output=Vec::with_capacity(depth.len()*latitude.len()*longitude.len());
        for &iz in &zi { for &iy in &yi { for &ix in &xi {
            let mut idx=vec![0usize;shape.len()];
            for (d,n) in names.iter().enumerate() { if n==&xdim {idx[d]=ix;} else if n==&ydim {idx[d]=iy;} else if n==&zdim {idx[d]=iz;} else if time_dim.as_ref()==Some(n) { ensure!(nc.time_index < shape[d], "time_index={} は変数 {name} の時刻数 {} を超えています", nc.time_index, shape[d]); idx[d]=nc.time_index; } else { ensure!(shape[d]==1,"物理変数 {name} に未対応の次元 {n} があります"); } }
            let mut value=values[flatten(&idx,&shape)]; ensure!(value.is_finite()&&value.abs()<1.0e20,"物理変数 {name} に欠損値があります");
            // HYCOM's archived NCSS response uses packed shorts with scale_factor
            // and add_offset attributes. netcdf::get_values reads the stored numeric
            // values, so explicitly unpack the known GLBy0.08 variables here.
            // Keep the -30000 fill marker intact so w estimation can exclude it and
            // the bathymetry mask can remove land/sub-bottom cells downstream.
            if (value + 30000.0).abs() < 0.5 {
                output.push(-30000.0_f32);
                continue;
            }
            match name.as_str() {
                "water_u" | "water_v" | "water_w" => value *= 0.001,
                "water_temp" | "salinity" => value = value * 0.001 + 20.0,
                _ => {
                    if key=="temperature" && value>150.0 { value-=273.15; }
                    if key=="salinity" && value>0.0 && value<0.1 { value*=1000.0; }
                }
            }
            output.push(value as f32);
        }}}
        write_f32(&raw.join(format!("{key}.bin")),&output)?;
        println!("[NetCDF] {key}: {} ({} values)",name,output.len());
    }
    let coordinates=serde_json::json!({"depth":depth,"latitude":latitude,"longitude":longitude});
    fs::write(raw.join("coordinates.json"),serde_json::to_vec(&coordinates)?)?;
    let meta=serde_json::json!({"source_terrain":terrain,"source_ocean":path,"sos_cache_signature":signature,"shape_depth_lat_lon":[zi.len(),yi.len(),xi.len()],"variables":"Rust native NetCDF reader","note":"w is estimated from horizontal u/v via incompressible continuity when source w is unavailable; shallowest depth is constrained to w=0. This is an estimate, not a source-provided or observed field. Known packed HYCOM variables are unpacked using their documented scale_factor/add_offset; -30000 fill markers are retained for masking."});
    fs::write(manifest,serde_json::to_vec_pretty(&meta)?)?;
    Ok(())
}
/// Read little-endian f32 arrays written by this preprocessing pipeline.
fn read_f32_array(path: &Path) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("前処理配列を読めません: {}", path.display()))?;
    ensure!(bytes.len() % 4 == 0, "{} のサイズが4バイトの倍数ではありません", path.display());
    Ok(bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
}

/// Estimate upward-positive vertical velocity from eastward/northward velocity.
/// Arrays are laid out [depth, latitude, longitude], depth is positive downward.
/// The surface-most available layer is assigned w=0 as a boundary condition;
/// w is then integrated downward from d(w)/d(depth)=horizontal_divergence.
/// This is a continuity-based estimate, not an observed/model-provided w field.
fn estimate_vertical_velocity(u: &[f32], v: &[f32], depth: &[f32], lat: &[f64], lon: &[f64]) -> Result<Vec<f32>> {
    let nz = depth.len(); let ny = lat.len(); let nx = lon.len();
    let count = nz.checked_mul(ny).and_then(|n| n.checked_mul(nx)).ok_or_else(|| anyhow!("w配列サイズがオーバーフローしました"))?;
    ensure!(u.len() == count && v.len() == count, "w推定に必要なu/v配列のサイズが一致しません");
    ensure!(nz >= 2 && ny >= 2 && nx >= 2, "w推定には深度・緯度・経度がそれぞれ2点以上必要です");
    ensure!(depth.windows(2).all(|p| p[1] > p[0]), "w推定用の深度は昇順である必要があります");
    ensure!(lat.windows(2).all(|p| p[1] > p[0]) && lon.windows(2).all(|p| p[1] > p[0]), "w推定用の緯度・経度は昇順である必要があります");
    let index = |k: usize, j: usize, i: usize| (k * ny + j) * nx + i;
    let valid = |x: f32| x.is_finite() && x.abs() < 20_000.0; // excludes HYCOM's packed -30000 fill value
    let mut divergence = vec![0.0f32; count];
    for k in 0..nz { for j in 0..ny { for i in 0..nx {
        let c = index(k,j,i);
        if !valid(u[c]) || !valid(v[c]) { continue; }
        let lat_rad = lat[j].to_radians();
        let dx_degrees = if i == 0 { lon[1]-lon[0] } else if i+1 == nx { lon[nx-1]-lon[nx-2] } else { (lon[i+1]-lon[i-1])*0.5 };
        let dy_degrees = if j == 0 { lat[1]-lat[0] } else if j+1 == ny { lat[ny-1]-lat[ny-2] } else { (lat[j+1]-lat[j-1])*0.5 };
        let dx = dx_degrees * 111_320.0 * lat_rad.cos().abs().max(0.01);
        let dy = dy_degrees * 110_540.0;
        let du_dx = if i == 0 {
            let a=index(k,j,0); let b=index(k,j,1); if valid(u[a]) && valid(u[b]) {(u[b]-u[a])/(dx as f32)} else {0.0}
        } else if i+1 == nx {
            let a=index(k,j,nx-2); let b=index(k,j,nx-1); if valid(u[a]) && valid(u[b]) {(u[b]-u[a])/(dx as f32)} else {0.0}
        } else {
            let a=index(k,j,i-1); let b=index(k,j,i+1); let span=(lon[i+1]-lon[i-1])*111_320.0*lat_rad.cos().abs().max(0.01); if valid(u[a]) && valid(u[b]) {(u[b]-u[a])/(span as f32)} else {0.0}
        };
        let dv_dy = if j == 0 {
            let a=index(k,0,i); let b=index(k,1,i); if valid(v[a]) && valid(v[b]) {(v[b]-v[a])/(dy as f32)} else {0.0}
        } else if j+1 == ny {
            let a=index(k,ny-2,i); let b=index(k,ny-1,i); if valid(v[a]) && valid(v[b]) {(v[b]-v[a])/(dy as f32)} else {0.0}
        } else {
            let a=index(k,j-1,i); let b=index(k,j+1,i); let span=(lat[j+1]-lat[j-1])*110_540.0; if valid(v[a]) && valid(v[b]) {(v[b]-v[a])/(span as f32)} else {0.0}
        };
        divergence[c] = du_dx + dv_dy;
    }}}
    let mut w = vec![0.0f32; count];
    // Boundary condition: w=0 at the shallowest available HYCOM layer.
    for k in 1..nz { let dz = depth[k] - depth[k-1]; for j in 0..ny { for i in 0..nx {
        let c=index(k,j,i); let p=index(k-1,j,i);
        w[c] = w[p] + 0.5 * (divergence[p] + divergence[c]) * dz;
    }}}
    ensure!(w.iter().all(|x| x.is_finite()), "w推定結果にNaN/Infinityが含まれています");
    Ok(w)
}

fn flatten(index:&[usize],shape:&[usize])->usize { index.iter().zip(shape).fold(0usize,|a,(i,n)|a*n+i) }
fn write_f32(path:&Path,values:&[f32])->Result<()> { let mut bytes=Vec::with_capacity(values.len()*4); for v in values {bytes.extend_from_slice(&v.to_le_bytes());} fs::write(path,bytes)?; Ok(()) }
fn validate_terrain(t:&TerrainJson)->Result<()> { ensure!(t.shape.len()==2&&t.shape[0]>=2&&t.shape[1]>=2,"地形格子は2×2以上必要です"); ensure!(t.longitude.len()==t.shape[1]&&t.latitude.len()==t.shape[0]&&t.elevation_m.len()==t.shape[0]*t.shape[1],"地形座標と配列サイズが一致しません"); ensure!(t.longitude.windows(2).all(|w|w[1]>w[0])&&t.latitude.windows(2).all(|w|w[1]>w[0]),"地形の経度・緯度は昇順である必要があります"); ensure!(t.elevation_m.iter().all(|v|v.is_finite())&&t.elevation_m.iter().any(|v|*v<0.0),"地形に欠損値があるか、海底標高の負値がありません"); Ok(()) }
fn average_step(a:&[f32])->f32 { if a.len()<2 {1.0} else {a.windows(2).map(|w|(w[1]-w[0]).abs()).sum::<f32>()/(a.len()-1) as f32} }
fn cache_signature(nc:&NetcdfInputConfig,goal:&str,terrain:&Path,ocean:&Path)->Result<String> { let mut h=Fnv64(0xcbf29ce484222325); h.write(&serde_json::to_vec(nc)?); h.write(goal.as_bytes()); h.write(b"vertical_velocity_estimator_v2_shared_hycom_domain"); for p in [terrain,ocean] {let m=fs::metadata(p)?;h.write(p.to_string_lossy().as_bytes());h.write(&m.len().to_le_bytes());let t=m.modified().unwrap_or(UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();h.write(&t.to_le_bytes());} Ok(format!("{:016x}",h.finish())) }
struct Fnv64(u64); impl Hasher for Fnv64 {fn finish(&self)->u64{self.0} fn write(&mut self,b:&[u8]){for v in b {self.0^=u64::from(*v);self.0=self.0.wrapping_mul(0x100000001b3);}}}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crop_axis_sorts_selected_coordinate_indices() {
        assert_eq!(crop_axis(&[139.0, 137.0, 138.0, 140.0], 137.5, 139.5, false), vec![2, 0]);
    }
    #[test]
    fn depth_axis_is_sorted_by_positive_down_magnitude() {
        assert_eq!(crop_axis(&[-100.0, 0.0, -20.0, -500.0], 0.0, 200.0, true), vec![1, 2, 0]);
    }
    #[test]
    fn row_major_flattening_matches_c_order() {
        assert_eq!(flatten(&[1, 2, 3], &[2, 4, 5]), 33);
    }

    #[test]
    fn terrain_clipping_uses_hycom_domain_and_preserves_row_major_values() {
        let terrain = TerrainJson {
            longitude: vec![137.0, 137.5, 138.0, 138.5, 139.0],
            latitude: vec![33.0, 33.5, 34.0, 34.5],
            elevation_m: (0..20).map(|v| -(v as f32)).collect(),
            shape: vec![4, 5],
        };
        let clipped = clip_terrain_to_bounds(terrain, (137.5, 138.5, 33.5, 34.5)).unwrap();
        assert_eq!(clipped.longitude, vec![137.5, 138.0, 138.5]);
        assert_eq!(clipped.latitude, vec![33.5, 34.0, 34.5]);
        assert_eq!(clipped.shape, vec![3, 3]);
        assert_eq!(clipped.elevation_m, vec![-5.0, -6.0, -7.0, -10.0, -11.0, -12.0, -15.0, -16.0, -17.0]);
    }

    #[test]
    fn terrain_clipping_rejects_non_overlapping_domains() {
        let terrain = TerrainJson {
            longitude: vec![137.0, 137.5], latitude: vec![33.0, 33.5],
            elevation_m: vec![-1.0, -2.0, -3.0, -4.0], shape: vec![2, 2],
        };
        assert!(clip_terrain_to_bounds(terrain, (138.0, 139.0, 34.0, 35.0)).is_err());
    }

    #[test]
    fn estimated_vertical_velocity_is_zero_for_uniform_horizontal_flow() {
        let depth = vec![1.0, 10.0, 100.0];
        let lat = vec![34.0, 34.5];
        let lon = vec![138.0, 138.5, 139.0];
        let n = depth.len() * lat.len() * lon.len();
        let w = estimate_vertical_velocity(&vec![1.0; n], &vec![0.5; n], &depth, &lat, &lon).unwrap();
        assert!(w.iter().all(|v| v.abs() < 1.0e-6));
    }

    #[test]
    fn estimated_vertical_velocity_rejects_mismatched_arrays() {
        let result = estimate_vertical_velocity(&[1.0; 8], &[1.0; 7], &[1.0, 10.0], &[34.0, 35.0], &[138.0, 139.0]);
        assert!(result.is_err());
    }
}
