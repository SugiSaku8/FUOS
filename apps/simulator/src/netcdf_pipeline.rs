//! Native Rust input pipeline: public NetCDF / geographic GeoTIFF -> normalized arrays -> s-sof.
//! No external interpreter or preprocessing scripts are used.
use anyhow::{Context, Result, ensure, anyhow};
use serde::{Deserialize, Serialize};
use s_field::Grid4D;
use s_sof::grid::GridWriter;
use std::{fs, path::{Path, PathBuf}, process::Command, time::UNIX_EPOCH, hash::Hasher};
use crate::{config::{Config, NetcdfInputConfig}, source_bundle::SourceBundle};

#[derive(Debug, Serialize, Deserialize)]
struct TerrainJson { longitude: Vec<f64>, latitude: Vec<f64>, elevation_m: Vec<f32>, shape: Vec<usize> }

pub fn is_configured(config: &Config) -> bool { config.input.netcdf.is_some() }

pub fn prepare(config: &Config, _config_path: &Path) -> Result<()> {
    let Some(nc) = config.input.netcdf.as_ref() else { return Ok(()); };
    let terrain_path = config.resolve(&nc.terrain_file);
    let ocean_path = config.resolve(&nc.ocean_file);
    ensure!(terrain_path.is_file(), "地形データがありません: {}。NetCDFまたはEPSG:4326 GeoTIFFを配置してください。", terrain_path.display());
    ensure!(ocean_path.is_file(), "海洋NetCDFがありません: {}。先に `cargo run -p simulator -- get --hycom` を実行してください。", ocean_path.display());
    let prepared = config.resolve(&nc.prepared_dir);
    let raw_dir = prepared.join("raw_arrays");
    fs::create_dir_all(&raw_dir)?;
    let manifest_path = prepared.join("preprocess_manifest.json");
    let terrain_json_path = prepared.join("terrain.json");
    let signature = cache_signature(nc, &terrain_path, &ocean_path)?;
    let required = ["coordinates.json", "u.bin", "v.bin", "temperature.bin", "salinity.bin"];
    let cached = manifest_path.is_file() && terrain_json_path.is_file() && required.iter().all(|n| raw_dir.join(n).is_file())
        && fs::read_to_string(&manifest_path).unwrap_or_default().contains(&format!("\"sos_cache_signature\": \"{signature}\""));
    if !cached {
        println!("[Rust NetCDF] 公開入力を前処理します: {} + {}", terrain_path.display(), ocean_path.display());
        let terrain = if is_geotiff(&terrain_path) { read_geotiff(&terrain_path, nc)? } else { read_terrain_netcdf(&terrain_path, nc)? };
        validate_terrain(&terrain)?;
        fs::write(&terrain_json_path, serde_json::to_vec(&terrain)?)?;
        read_ocean_netcdf(&ocean_path, nc, &raw_dir, &manifest_path, &terrain_path, &signature)?;
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
    let terrain_target = config.input.terrain_source.as_ref().or(config.input.terrain_grid.as_ref()).ok_or_else(|| anyhow!("input.terrain_source を設定してください"))?;
    let terrain_snc = config.resolve(terrain_target);
    if let Some(p) = terrain_snc.parent() { fs::create_dir_all(p)?; }
    let mut writer = GridWriter::create(&terrain_snc).context("s-sof GridWriterを作成できません")?;
    writer.write(&grid, &lat, &lon, &[0.0], &terrain.elevation_m).context("terrain SNCへの書き込みに失敗しました")?;
    writer.finish()?;
    println!("[s-sof] 地形 → {} ({}×{})", terrain_snc.display(), w, h);
    let bundle = SourceBundle::from_legacy_dir(&raw_dir, true, true, true)?;
    let bundle_target = config.input.source_bundle.as_ref().ok_or_else(|| anyhow!("input.source_bundle が必要です"))?;
    let bundle_path = config.resolve(bundle_target);
    bundle.write(&bundle_path)?;
    println!("[Rust NetCDF] 物理場 → {}", bundle_path.display());
    Ok(())
}

pub fn download_hycom(config: &Config, force: bool) -> Result<()> {
    let nc = config.input.netcdf.as_ref().ok_or_else(|| anyhow!("HYCOM取得には input.netcdf 設定が必要です"))?;
    let output = config.resolve(&nc.ocean_file);
    if output.is_file() && fs::metadata(&output)?.len() > 1024 && !force {
        println!("[HYCOM] 既存データを再利用: {} (再取得は --force)", output.display()); return Ok(());
    }
    let base = nc.hycom_url.as_deref().unwrap_or("https://ncss.hycom.org/thredds/ncss/GLBy0.08/expt_93.0");
    let base = base.replace("/thredds/dodsC/", "/thredds/ncss/").replace("/dodsC/", "/ncss/");
    let r = &nc.region;
    let query = format!("var=water_temp&var=salinity&var=water_u&var=water_v&north={}&west={}&east={}&south={}&disableProjSubset=on&time=last&accept=netcdf", r.latitude_max, r.longitude_min, r.longitude_max, r.latitude_min);
    let url = format!("{}?{}", base.trim_end_matches('?'), query);
    if let Some(p) = output.parent() { fs::create_dir_all(p)?; }
    let tmp = output.with_extension("nc.download.tmp");
    println!("[HYCOM] 公開NCSSから対象領域を取得します: {}", url);
    let status = Command::new("curl").args(["--fail", "--location", "--silent", "--show-error", "--retry", "3", "--output"])
        .arg(&tmp).arg(&url).status().context("curlを起動できません。macOS標準環境またはHomebrewのcurlを確認してください")?;
    ensure!(status.success(), "HYCOM取得に失敗しました。ネットワーク、公開NCSSの稼働状況、hycom_urlを確認してください");
    ensure!(fs::metadata(&tmp)?.len() > 1024, "HYCOMサーバーの応答がNetCDFとして小さすぎます");
    // Check the response is a readable NetCDF before replacing a valid previous file.
    netcdf::open(&tmp).context("HYCOM取得結果をNetCDFとして開けません。NCSS endpoint/queryを確認してください")?;
    fs::rename(&tmp, &output).with_context(|| format!("取得ファイルを保存できません: {}", output.display()))?;
    println!("[HYCOM] 保存完了: {} ({} bytes)", output.display(), fs::metadata(&output)?.len());
    Ok(())
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
    let config_keys: &[&str] = match key {
        "longitude" => &["longitude", "ocean_longitude", "terrain_longitude"],
        "latitude" => &["latitude", "ocean_latitude", "terrain_latitude"],
        "terrain" => &["terrain", "terrain_elevation"],
        other => std::slice::from_ref(&other),
    };
    for config_key in config_keys {
        if let Some(v) = nc.variables.get(*config_key) {
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
    use gdal::Dataset;
    let ds = Dataset::open(path).with_context(|| format!("GeoTIFFを開けません: {}", path.display()))?;
    ensure!(ds.spatial_ref()?.is_geographic(), "GeoTIFFは地理座標系が必要です。事前にEPSG:4326へ変換してください");
    let gt = ds.geo_transform()?; ensure!(gt[1] > 0.0 && gt[5] < 0.0 && gt[2].abs() < 1e-12 && gt[4].abs() < 1e-12, "GeoTIFFは回転のない北向き規則格子が必要です");
    let (width,height) = ds.raster_size(); let r = &nc.region;
    let x0 = (((r.longitude_min-gt[0])/gt[1]).floor() as isize).clamp(0,width as isize) as usize;
    let x1 = (((r.longitude_max-gt[0])/gt[1]).ceil() as isize).clamp(0,width as isize) as usize;
    let y0 = (((r.latitude_max-gt[3])/gt[5]).floor() as isize).clamp(0,height as isize) as usize;
    let y1 = (((r.latitude_min-gt[3])/gt[5]).ceil() as isize).clamp(0,height as isize) as usize;
    ensure!(x1>x0+1 && y1>y0+1, "GeoTIFFに指定海域の2×2以上の画素がありません");
    let band = ds.rasterband(1)?; let nodata = band.no_data_value();
    let buf: gdal::raster::Buffer<f64> = band.read_as((x0 as isize,y0 as isize),(x1-x0,y1-y0),(x1-x0,y1-y0),None)?;
    let longitude = (x0..x1).map(|x| gt[0]+(x as f64+0.5)*gt[1]).collect::<Vec<_>>();
    let latitude = (y0..y1).rev().map(|y| gt[3]+(y as f64+0.5)*gt[5]).collect::<Vec<_>>();
    let mut elevation = Vec::with_capacity((x1-x0)*(y1-y0));
    for row in (0..(y1-y0)).rev() { for col in 0..(x1-x0) { let v=buf.data()[row*(x1-x0)+col]; ensure!(v.is_finite() && v.abs()<1.0e20 && nodata.map(|n| (v-n).abs() > f64::EPSILON).unwrap_or(true), "GeoTIFFにNoData/欠損値があります"); elevation.push(v as f32); } }
    Ok(TerrainJson { longitude, latitude, elevation_m:elevation, shape:vec![y1-y0,x1-x0] })
}
fn read_ocean_netcdf(path: &Path, nc: &NetcdfInputConfig, raw: &Path, manifest: &Path, terrain: &Path, signature: &str) -> Result<()> {
    let file = netcdf::open(path).with_context(|| format!("海洋NetCDFを開けません: {}", path.display()))?;
    let (_, xdim, xall)=axis(&file,nc,"longitude")?; let (_, ydim, yall)=axis(&file,nc,"latitude")?; let (_, zdim,zall)=axis(&file,nc,"depth")?;
    let r=&nc.region; let xi=crop_axis(&xall,r.longitude_min,r.longitude_max,false); let yi=crop_axis(&yall,r.latitude_min,r.latitude_max,false); let zi=crop_axis(&zall,0.0,nc.hycom_max_depth_m,true);
    ensure!(xi.len()>=2 && yi.len()>=2 && zi.len()>=2,"海洋NetCDFの対象領域に2×2×2以上の格子がありません");
    let mut depth:Vec<f32>=zi.iter().map(|i|zall[*i].abs() as f32).collect();
    ensure!(depth.windows(2).all(|w|w[1]>w[0]),"深度座標が昇順または一意ではありません");
    let longitude:Vec<f64>=xi.iter().map(|i|xall[*i]).collect(); let latitude:Vec<f64>=yi.iter().map(|i|yall[*i]).collect();
    let time_name=var_name(&file,nc,"time",false)?;
    let time_dim=time_name.as_ref().and_then(|n|file.variable(n)).and_then(|v|v.dimensions().first().map(|d|d.name().to_string()));
    // Do not let an optional vertical-velocity field from an older input leak into a new bundle.
    let _ = fs::remove_file(raw.join("w.bin"));
    for key in ["u","v","w","temperature","salinity"] {
        let Some(name)=var_name(&file,nc,key,key!="w")? else { continue; };
        let var=file.variable(&name).unwrap(); let dims=var.dimensions(); let shape:Vec<usize>=dims.iter().map(|d|d.len()).collect(); let names:Vec<String>=dims.iter().map(|d|d.name()).collect();
        let values=var.get_values::<f64,_>(..).with_context(||format!("物理変数 {name} を読み込めません"))?;
        let mut output=Vec::with_capacity(depth.len()*latitude.len()*longitude.len());
        for &iz in &zi { for &iy in &yi { for &ix in &xi {
            let mut idx=vec![0usize;shape.len()];
            for (d,n) in names.iter().enumerate() { if n==&xdim {idx[d]=ix;} else if n==&ydim {idx[d]=iy;} else if n==&zdim {idx[d]=iz;} else if time_dim.as_ref()==Some(n) { ensure!(nc.time_index < shape[d], "time_index={} は変数 {name} の時刻数 {} を超えています", nc.time_index, shape[d]); idx[d]=nc.time_index; } else { ensure!(shape[d]==1,"物理変数 {name} に未対応の次元 {n} があります"); } }
            let mut v=values[flatten(&idx,&shape)]; ensure!(v.is_finite()&&v.abs()<1.0e20,"物理変数 {name} に欠損値があります");
            if ["u","v","w"].contains(&key) { /* Common HYCOM velocities are m/s; packed-unit conversion must be explicit in metadata. */ }
            if key=="temperature" && v>150.0 { v-=273.15; }
            if key=="salinity" && v>0.0 && v<0.1 { v*=1000.0; }
            output.push(v as f32);
        }}}
        write_f32(&raw.join(format!("{key}.bin")),&output)?;
        println!("[NetCDF] {key}: {} ({} values)",name,output.len());
    }
    let coordinates=serde_json::json!({"depth":depth,"latitude":latitude,"longitude":longitude});
    fs::write(raw.join("coordinates.json"),serde_json::to_vec(&coordinates)?)?;
    let meta=serde_json::json!({"source_terrain":terrain,"source_ocean":path,"sos_cache_signature":signature,"shape_depth_lat_lon":[zi.len(),yi.len(),xi.len()],"variables":"Rust native NetCDF reader","note":"Missing values are rejected; units are normalized only by safe heuristic conversions."});
    fs::write(manifest,serde_json::to_vec_pretty(&meta)?)?;
    Ok(())
}
fn flatten(index:&[usize],shape:&[usize])->usize { index.iter().zip(shape).fold(0usize,|a,(i,n)|a*n+i) }
fn write_f32(path:&Path,values:&[f32])->Result<()> { let mut bytes=Vec::with_capacity(values.len()*4); for v in values {bytes.extend_from_slice(&v.to_le_bytes());} fs::write(path,bytes)?; Ok(()) }
fn validate_terrain(t:&TerrainJson)->Result<()> { ensure!(t.shape.len()==2&&t.shape[0]>=2&&t.shape[1]>=2,"地形格子は2×2以上必要です"); ensure!(t.longitude.len()==t.shape[1]&&t.latitude.len()==t.shape[0]&&t.elevation_m.len()==t.shape[0]*t.shape[1],"地形座標と配列サイズが一致しません"); ensure!(t.longitude.windows(2).all(|w|w[1]>w[0])&&t.latitude.windows(2).all(|w|w[1]>w[0]),"地形の経度・緯度は昇順である必要があります"); ensure!(t.elevation_m.iter().all(|v|v.is_finite())&&t.elevation_m.iter().any(|v|*v<0.0),"地形に欠損値があるか、海底標高の負値がありません"); Ok(()) }
fn average_step(a:&[f32])->f32 { if a.len()<2 {1.0} else {a.windows(2).map(|w|(w[1]-w[0]).abs()).sum::<f32>()/(a.len()-1) as f32} }
fn cache_signature(nc:&NetcdfInputConfig,terrain:&Path,ocean:&Path)->Result<String> { let mut h=Fnv64(0xcbf29ce484222325); h.write(&serde_json::to_vec(nc)?); for p in [terrain,ocean] {let m=fs::metadata(p)?;h.write(p.to_string_lossy().as_bytes());h.write(&m.len().to_le_bytes());let t=m.modified().unwrap_or(UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();h.write(&t.to_le_bytes());} Ok(format!("{:016x}",h.finish())) }
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
}
