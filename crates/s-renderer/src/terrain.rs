use crate::{
    coordinate::{GeoBounds, GeoConverter},
    heightmap::HeightMap,
    mesh::{GpuMesh, Mesh},
    terrain_mesh::TerrainVertex,
};

/// 海底地形のCPU側メッシュ。
#[derive(Debug)]
pub struct TerrainMesh {
    mesh: Mesh<TerrainVertex>,
}

impl TerrainMesh {
    /// HeightMapと地理的な境界から地形メッシュを生成する。
    ///
    /// vertical_scaleは高さ方向の倍率。
    pub fn from_heightmap(heightmap: &HeightMap, bounds: GeoBounds, vertical_scale: f32) -> Self {
        assert!(heightmap.width >= 2);
        assert!(heightmap.height >= 2);

        assert_eq!(heightmap.data.len(), heightmap.width * heightmap.height);

        assert!(vertical_scale.is_finite());

        let converter = GeoConverter::new(bounds, 100_000.0);

        let mut vertices = Vec::with_capacity(heightmap.width * heightmap.height);

        for z in 0..heightmap.height {
            let latitude = bounds.lat_min
                + (bounds.lat_max - bounds.lat_min) * z as f64 / (heightmap.height - 1) as f64;

            for x in 0..heightmap.width {
                let longitude = bounds.lon_min
                    + (bounds.lon_max - bounds.lon_min) * x as f64 / (heightmap.width - 1) as f64;

                let height = heightmap.get(x, z);

                let mut position = converter.ocean_to_world(longitude, latitude, 0.0);

                // 海底の高さをワールド座標に反映する。
                position[1] = height * vertical_scale;

                vertices.push(TerrainVertex { position, height });
            }
        }

        let mut indices = Vec::with_capacity((heightmap.width - 1) * (heightmap.height - 1) * 6);

        for z in 0..heightmap.height - 1 {
            for x in 0..heightmap.width - 1 {
                let top_left = (z * heightmap.width + x) as u32;

                let top_right = top_left + 1;

                let bottom_left = top_left + heightmap.width as u32;

                let bottom_right = bottom_left + 1;

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

    pub fn vertex_count(&self) -> usize {
        self.mesh.vertex_count()
    }

    pub fn index_count(&self) -> usize {
        self.mesh.index_count()
    }

    pub fn upload(&self, device: &wgpu::Device) -> GpuMesh<TerrainVertex> {
        GpuMesh::upload(device, &self.mesh)
    }
}

/// 海面のCPU側メッシュ。
///
/// 海底地形とは独立した水平面として生成する。
/// 描画時には専用の半透明パイプラインを使用する。
#[derive(Debug)]
pub struct OceanSurfaceMesh {
    mesh: Mesh<TerrainVertex>,
}

impl OceanSurfaceMesh {
    /// 指定された範囲に水平な海面を生成する。
    ///
    /// width、heightは海面の格子数。
    pub fn new(bounds: GeoBounds, width: usize, height: usize) -> Self {
        assert!(width >= 2);
        assert!(height >= 2);

        let converter = GeoConverter::new(bounds, 100_000.0);

        let mut vertices = Vec::with_capacity(width * height);

        for z in 0..height {
            let latitude =
                bounds.lat_min + (bounds.lat_max - bounds.lat_min) * z as f64 / (height - 1) as f64;

            for x in 0..width {
                let longitude = bounds.lon_min
                    + (bounds.lon_max - bounds.lon_min) * x as f64 / (width - 1) as f64;

                let position = converter.ocean_to_world(longitude, latitude, 0.0);

                vertices.push(TerrainVertex {
                    position,
                    height: 0.0,
                });
            }
        }

        let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);

        for z in 0..height - 1 {
            for x in 0..width - 1 {
                let top_left = (z * width + x) as u32;

                let top_right = top_left + 1;

                let bottom_left = top_left + width as u32;

                let bottom_right = bottom_left + 1;

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

    pub fn vertex_count(&self) -> usize {
        self.mesh.vertex_count()
    }

    pub fn index_count(&self) -> usize {
        self.mesh.index_count()
    }

    /// 海面メッシュをGPUへ転送する。
    pub fn upload(&self, device: &wgpu::Device) -> GpuMesh<TerrainVertex> {
        GpuMesh::upload(device, &self.mesh)
    }
}
