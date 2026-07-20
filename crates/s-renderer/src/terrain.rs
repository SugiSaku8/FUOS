use std::mem;

use bytemuck::{Pod, Zeroable};

use crate::mesh::{GpuMesh, Mesh};

//
// GPUへ送る地形頂点
//
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],
}

//
// Vertex属性
// staticに保持する
//
const TERRAIN_ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
    0 => Float32x3,
    1 => Float32x3
];

impl TerrainVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &TERRAIN_ATTRIBUTES,
        }
    }
}

//
// CPU側 Terrain Mesh
//
pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,

    pub indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn new(size: usize) -> Self {
        let mut vertices = Vec::new();

        let mut indices = Vec::new();

        //
        // 簡易地形生成
        //
        for z in 0..=size {
            for x in 0..=size {
                let xf = x as f32 / size as f32;

                let zf = z as f32 / size as f32;

                let height = (xf * 10.0).sin() * (zf * 10.0).cos() * 3.0;

                vertices.push(TerrainVertex {
                    position: [x as f32, height, z as f32],

                    normal: [0.0, 1.0, 0.0],
                });
            }
        }

        //
        // index生成
        //
        for z in 0..size {
            for x in 0..size {
                let i = z * (size + 1) + x;

                indices.push(i as u32);
                indices.push((i + 1) as u32);
                indices.push((i + size + 1) as u32);

                indices.push((i + 1) as u32);
                indices.push((i + size + 2) as u32);
                indices.push((i + size + 1) as u32);
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
