use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{
    coordinate::{GeoBounds, GeoConverter},
    heightmap::HeightMap,
    mesh::{GpuMesh, Mesh},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],

    //
    // 高度
    //
    pub height: f32,
}

impl TerrainVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: ATTRIBUTES,
        }
    }
}

pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,

    pub indices: Vec<u32>,
}

impl TerrainMesh {
    //
    // GEBCO HeightMapから生成
    //
    pub fn from_heightmap(heightmap: &HeightMap, bounds: GeoBounds) -> Self {
        let converter = GeoConverter::new(bounds);

        let mut vertices = Vec::new();

        let mut indices = Vec::new();

        let width = heightmap.width;

        let height = heightmap.height;

        //
        // vertices
        //
        for z in 0..height {
            let lat = bounds.lat_min
                + (z as f64 / (height - 1) as f64) * (bounds.lat_max - bounds.lat_min);

            for x in 0..width {
                let lon = bounds.lon_min
                    + (x as f64 / (width - 1) as f64) * (bounds.lon_max - bounds.lon_min);

                let elevation = heightmap.get(x, z);

                let position = converter.convert(lon, lat, elevation);

                vertices.push(TerrainVertex {
                    position,

                    height: elevation,
                });
            }
        }

        //
        // indices
        //
        for z in 0..height - 1 {
            for x in 0..width - 1 {
                let i = (z * width + x) as u32;

                indices.push(i);

                indices.push(i + 1);

                indices.push(i + width as u32);

                indices.push(i + 1);

                indices.push(i + width as u32 + 1);

                indices.push(i + width as u32);
            }
        }

        Self { vertices, indices }
    }

    //
    // GPUへアップロード
    //
    pub fn upload(&self, device: &wgpu::Device) -> GpuMesh<TerrainVertex> {
        let mesh = Mesh {
            vertices: self.vertices.clone(),

            indices: self.indices.clone(),
        };

        GpuMesh::upload(device, &mesh)
    }
}
