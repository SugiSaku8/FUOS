use crate::{
    coordinate::{GeoBounds, GeoConverter},
    heightmap::HeightMap,
    mesh::{GpuMesh, Mesh},
    terrain_mesh::TerrainVertex,
};

/// 海底地形のCPU側メッシュ。
///
/// HeightMapから頂点とインデックスを生成し、
/// 既存のGpuMeshを使ってGPUへアップロードする。
#[derive(Debug)]
pub struct TerrainMesh {
    mesh: Mesh<TerrainVertex>,
}

impl TerrainMesh {
    /// HeightMapと地理的な境界から地形メッシュを生成する。
    ///
    /// vertical_scale は高さ方向の倍率。
    /// 経度・緯度方向の座標変換はGeoConverterに任せる。
    pub fn from_heightmap(heightmap: &HeightMap, bounds: GeoBounds, vertical_scale: f32) -> Self {
        assert!(heightmap.width >= 2, "HeightMap width must be >= 2");
        assert!(heightmap.height >= 2, "HeightMap height must be >= 2");
        assert!(
            heightmap.data.len() == heightmap.width * heightmap.height,
            "HeightMap data length does not match dimensions"
        );
        assert!(vertical_scale.is_finite(), "vertical_scale must be finite");

        let converter = GeoConverter::new(bounds, 100_000.0);
        let mut vertices = Vec::with_capacity(heightmap.width * heightmap.height);
        for z in 0..heightmap.height {
            let latitude = if heightmap.height > 1 {
                bounds.lat_min
                    + (bounds.lat_max - bounds.lat_min) * z as f64 / (heightmap.height - 1) as f64
            } else {
                bounds.lat_min
            };
            for x in 0..heightmap.width {
                let longitude = if heightmap.width > 1 {
                    bounds.lon_min
                        + (bounds.lon_max - bounds.lon_min) * x as f64
                            / (heightmap.width - 1) as f64
                } else {
                    bounds.lon_min
                };
                let height = heightmap.get(x, z);
                let mut position = converter.ocean_to_world(longitude, latitude, 0.0);
                // 高さのスケールを適用する。
                // 海底の負の値はワールド座標の下方向に配置される想定。
                position[1] = height * vertical_scale;
                vertices.push(TerrainVertex { position, height });
            }
        }
        let mut indices = Vec::with_capacity((heightmap.width - 1) * (heightmap.height - 1) * 6);
        for z in 0..(heightmap.height - 1) {
            for x in 0..(heightmap.width - 1) {
                let top_left = (z * heightmap.width + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = top_left + heightmap.width as u32;
                let bottom_right = bottom_left + 1;
                // 1つの格子を2つの三角形に分割する。
                indices.extend_from_slice(&[
                    top_left,
                    bottom_left,
                    top_right,
                    top_right,
                    bottom_left,
                    bottom_right,
                ]);
            }
        }
        Self {
            mesh: Mesh::new(vertices, indices),
        }
    }
    /// CPU側の頂点数を取得する。
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertex_count()
    }
    /// CPU側のインデックス数を取得する。
    pub fn index_count(&self) -> usize {
        self.mesh.index_count()
    }
    /// 地形メッシュをGPUへアップロードする。
    pub fn upload(&self, device: &wgpu::Device) -> GpuMesh<TerrainVertex> {
        GpuMesh::upload(device, &self.mesh)
    }
}
