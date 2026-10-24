//! Compact, self-contained source-data bundle used by the SOS data-generation pipeline.
//!
//! File layout: 8-byte magic (`SOSSRC02`; reader also accepts `SOSSRC01`), little-endian u32 JSON-header length,
//! UTF-8 JSON header, then selected f32 arrays in the fixed order u, v, optional w, temperature,
//! salinity. All values are little-endian. The coordinates and field-presence flags
//! live in the header, so a bundle is a single input file.

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

const MAGIC_V1: &[u8; 8] = b"SOSSRC01";
const MAGIC_V2: &[u8; 8] = b"SOSSRC02";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceCoordinates {
    pub depth: Vec<f32>,
    pub latitude: Vec<f64>,
    pub longitude: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Header {
    version: u32,
    coordinates: SourceCoordinates,
    has_velocity: bool,
    #[serde(default)]
    has_vertical_velocity: bool,
    has_temperature: bool,
    has_salinity: bool,
    source_count: usize,
}

#[derive(Clone, Debug)]
pub struct SourceBundle {
    pub coordinates: SourceCoordinates,
    pub velocity_u: Option<Vec<f32>>,
    pub velocity_v: Option<Vec<f32>>,
    pub velocity_w: Option<Vec<f32>>,
    pub temperature: Option<Vec<f32>>,
    pub salinity: Option<Vec<f32>>,
}

impl SourceBundle {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).with_context(|| format!("ソースバンドルを読めません: {}", path.display()))?;
        ensure!(bytes.len() >= 12, "ソースバンドルが短すぎます: {}", path.display());
        let magic = bytes.get(..8).ok_or_else(|| anyhow::anyhow!("ソースバンドルの識別子がありません"))?;
        let version2 = magic == &MAGIC_V2[..];
        ensure!(version2 || magic == &MAGIC_V1[..], "ソースバンドルの識別子が違います: {}", path.display());
        let header_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        ensure!(12 + header_len <= bytes.len(), "ソースバンドルのヘッダーが破損しています");
        let header: Header = serde_json::from_slice(&bytes[12..12 + header_len])
            .context("ソースバンドルのヘッダーJSONが不正です")?;
        ensure!(header.version == (if version2 { 2 } else { 1 }), "ソースバンドルの形式識別子とバージョンが一致しません: {}", header.version);
        validate_coordinates(&header.coordinates)?;
        let expected_count = header.coordinates.depth.len()
            .checked_mul(header.coordinates.latitude.len())
            .and_then(|n| n.checked_mul(header.coordinates.longitude.len()))
            .ok_or_else(|| anyhow::anyhow!("元データのサイズがオーバーフローしました"))?;
        ensure!(header.source_count == expected_count,
            "ヘッダーの要素数が座標と一致しません: {} != {}", header.source_count, expected_count);
        let mut offset = 12 + header_len;
        let mut read_field = |present: bool, name: &str| -> Result<Option<Vec<f32>>> {
            if !present { return Ok(None); }
            let byte_len = header.source_count.checked_mul(4)
                .ok_or_else(|| anyhow::anyhow!("{} データサイズがオーバーフローしました", name))?;
            ensure!(offset + byte_len <= bytes.len(), "{} データが途中で切れています", name);
            let values = bytes[offset..offset + byte_len].chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect::<Vec<_>>();
            ensure!(values.iter().all(|v| v.is_finite()), "{} データに NaN / Infinity が含まれています", name);
            offset += byte_len;
            Ok(Some(values))
        };
        let velocity_u = read_field(header.has_velocity, "velocity_u")?;
        let velocity_v = read_field(header.has_velocity, "velocity_v")?;
        let velocity_w = if version2 { read_field(header.has_vertical_velocity, "velocity_w")? } else { None };
        let temperature = read_field(header.has_temperature, "temperature")?;
        let salinity = read_field(header.has_salinity, "salinity")?;
        ensure!(offset == bytes.len(), "ソースバンドル末尾に未解釈のデータがあります");
        Ok(Self { coordinates: header.coordinates, velocity_u, velocity_v, velocity_w, temperature, salinity })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        validate_coordinates(&self.coordinates)?;
        let source_count = self.coordinates.depth.len()
            .checked_mul(self.coordinates.latitude.len())
            .and_then(|n| n.checked_mul(self.coordinates.longitude.len()))
            .ok_or_else(|| anyhow::anyhow!("元データのサイズがオーバーフローしました"))?;
        ensure!(self.velocity_u.is_some() == self.velocity_v.is_some(), "u と v はセットで指定してください");
        ensure!(self.velocity_w.is_none() || self.velocity_u.is_some(), "w は u/v とセットで指定してください");
        for (name, field) in [
            ("velocity_u", self.velocity_u.as_ref()),
            ("velocity_v", self.velocity_v.as_ref()),
            ("velocity_w", self.velocity_w.as_ref()),
            ("temperature", self.temperature.as_ref()),
            ("salinity", self.salinity.as_ref()),
        ] {
            if let Some(values) = field {
                ensure!(values.len() == source_count, "{} の要素数が不正です: {} != {}", name, values.len(), source_count);
                ensure!(values.iter().all(|v| v.is_finite()), "{} に NaN / Infinity が含まれています", name);
            }
        }
        let header = Header {
            version: 2,
            coordinates: self.coordinates.clone(),
            has_velocity: self.velocity_u.is_some(),
            has_vertical_velocity: self.velocity_w.is_some(),
            has_temperature: self.temperature.is_some(),
            has_salinity: self.salinity.is_some(),
            source_count,
        };
        let json = serde_json::to_vec(&header)?;
        let header_len = u32::try_from(json.len()).context("バンドルヘッダーが大きすぎます")?;
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) { fs::create_dir_all(parent)?; }
        let mut file = fs::File::create(path).with_context(|| format!("出力ファイルを作成できません: {}", path.display()))?;
        file.write_all(MAGIC_V2)?;
        file.write_all(&header_len.to_le_bytes())?;
        file.write_all(&json)?;
        for field in [&self.velocity_u, &self.velocity_v, &self.velocity_w, &self.temperature, &self.salinity] {
            if let Some(values) = field {
                for value in values { file.write_all(&value.to_le_bytes())?; }
            }
        }
        file.sync_all()?;
        Ok(())
    }

    pub fn from_legacy_dir(
        root: impl AsRef<Path>, include_velocity: bool, include_temperature: bool, include_salinity: bool,
    ) -> Result<Self> {
        let root = root.as_ref();
        let coordinates_path = root.join("coordinates.json");
        let coordinates: SourceCoordinates = serde_json::from_slice(&fs::read(&coordinates_path)
            .with_context(|| format!("座標ファイルを読めません: {}", coordinates_path.display()))?)
            .context("coordinates.json の形式が不正です")?;
        validate_coordinates(&coordinates)?;
        let count = coordinates.depth.len() * coordinates.latitude.len() * coordinates.longitude.len();
        let velocity_u = if include_velocity { Some(read_f32(&root.join("u.bin"), count)?) } else { None };
        let velocity_v = if include_velocity { Some(read_f32(&root.join("v.bin"), count)?) } else { None };
        let velocity_w_path = root.join("w.bin");
        let velocity_w = if include_velocity && velocity_w_path.is_file() { Some(read_f32(&velocity_w_path, count)?) } else { None };
        let temperature = if include_temperature { Some(read_f32(&root.join("temperature.bin"), count)?) } else { None };
        let salinity = if include_salinity { Some(read_f32(&root.join("salinity.bin"), count)?) } else { None };
        Ok(Self { coordinates, velocity_u, velocity_v, velocity_w, temperature, salinity })
    }
}

fn read_f32(path: &Path, expected: usize) -> Result<Vec<f32>> {
    let bytes = fs::read(path).with_context(|| format!("元データを読めません: {}", path.display()))?;
    ensure!(bytes.len() % 4 == 0, "{} のサイズが4バイトの倍数ではありません", path.display());
    let values = bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect::<Vec<_>>();
    ensure!(values.len() == expected, "{} の要素数が不正です: {} != {}", path.display(), values.len(), expected);
    ensure!(values.iter().all(|v| v.is_finite()), "{} に NaN / Infinity が含まれています", path.display());
    Ok(values)
}

fn validate_coordinates(c: &SourceCoordinates) -> Result<()> {
    ensure!(c.depth.len() >= 2, "元データには深度が2層以上必要です");
    ensure!(c.latitude.len() >= 2 && c.longitude.len() >= 2, "元データの緯度・経度は各2点以上必要です");
    ensure!(c.depth.iter().all(|v| v.is_finite()) && c.latitude.iter().all(|v| v.is_finite()) && c.longitude.iter().all(|v| v.is_finite()), "座標に NaN / Infinity が含まれています");
    ensure!(c.depth.windows(2).all(|w| w[1] > w[0]), "深度は昇順である必要があります");
    ensure!(c.latitude.windows(2).all(|w| w[1] > w[0]), "緯度は昇順である必要があります");
    ensure!(c.longitude.windows(2).all(|w| w[1] > w[0]), "経度は昇順である必要があります");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    #[test]
    fn round_trip_bundle() {
        let bundle = SourceBundle {
            coordinates: SourceCoordinates { depth: vec![0.0, 10.0], latitude: vec![34.0, 35.0], longitude: vec![138.0, 139.0] },
            velocity_u: Some(vec![1.0; 8]), velocity_v: Some(vec![2.0; 8]), velocity_w: Some(vec![3.0; 8]),
            temperature: Some(vec![15.0; 8]), salinity: None,
        };
        let path: PathBuf = std::env::temp_dir().join(format!("sos-source-bundle-{}.sosdata", std::process::id()));
        bundle.write(&path).unwrap();
        let read = SourceBundle::read(&path).unwrap();
        assert_eq!(read.velocity_u.unwrap(), vec![1.0; 8]);
        assert_eq!(read.velocity_v.unwrap(), vec![2.0; 8]);
        assert_eq!(read.velocity_w.unwrap(), vec![3.0; 8]);
        assert_eq!(read.temperature.unwrap(), vec![15.0; 8]);
        assert!(read.salinity.is_none());
        fs::remove_file(path).unwrap();
    }
}
