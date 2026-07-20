use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TestVertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],
}

impl TestVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,

                    shader_location: 0,

                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as u64,

                    shader_location: 1,

                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
use crate::mesh::Mesh;

pub fn create_triangle() -> Mesh<TestVertex> {
    let vertices = vec![
        TestVertex {
            position: [0.0, 0.8, 0.0],

            normal: [0.0, 0.0, 1.0],
        },
        TestVertex {
            position: [-0.8, -0.8, 0.0],

            normal: [0.0, 0.0, 1.0],
        },
        TestVertex {
            position: [0.8, -0.8, 0.0],

            normal: [0.0, 0.0, 1.0],
        },
    ];

    let indices = vec![0, 1, 2];

    Mesh::new(vertices, indices)
}
