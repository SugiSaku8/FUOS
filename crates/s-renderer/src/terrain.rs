use crate::mesh::{GpuMesh, Mesh};

use wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub uv: [f32; 2],
}

impl TerrainVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,

                    shader_location: 0,

                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,

                    shader_location: 1,

                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 3]>() + mem::size_of::<[f32; 3]>())
                        as wgpu::BufferAddress,

                    shader_location: 2,

                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,

    pub indices: Vec<u32>,
}

impl TerrainMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),

            indices: Vec::new(),
        }
    }

    pub fn from_heightmap(width: usize, height: usize, heights: &[f32], scale: f32) -> Self {
        let mut vertices = Vec::with_capacity(width * height);

        let mut indices = Vec::new();

        //
        // vertices
        //

        for z in 0..height {
            for x in 0..width {
                let h = heights[z * width + x];

                let px = x as f32 * scale;

                let pz = z as f32 * scale;

                vertices.push(TerrainVertex {
                    position: [px, h, pz],

                    normal: [0.0, 1.0, 0.0],

                    uv: [x as f32 / width as f32, z as f32 / height as f32],
                });
            }
        }

        //
        // indices
        //

        for z in 0..height - 1 {
            for x in 0..width - 1 {
                let i = (z * width + x) as u32;

                indices.extend_from_slice(&[
                    i,
                    i + 1,
                    i + width as u32,
                    i + 1,
                    i + width as u32 + 1,
                    i + width as u32,
                ]);
            }
        }

        Self { vertices, indices }
    }

    pub fn upload(&self, device: &wgpu::Device) -> GpuMesh<TerrainVertex> {
        let mesh = Mesh {
            vertices: self.vertices.clone(),

            indices: self.indices.clone(),
        };

        GpuMesh::upload(device, &mesh)
    }
}
