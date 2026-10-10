use crate::velocity::VelocityVertex;
use wgpu::util::DeviceExt;

pub struct GpuVelocityMesh {
    vertex_buffer: wgpu::Buffer,

    vertex_count: u32,
}

impl GpuVelocityMesh {
    pub fn new(device: &wgpu::Device, vertices: &[VelocityVertex]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("velocity_vertex_buffer"),

            contents: bytemuck::cast_slice(vertices),

            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,

            vertex_count: vertices.len() as u32,
        }
    }

    pub fn from_field(
        device: &wgpu::Device,

        velocity: &s_field::VelocityField,

        stride: usize,
    ) -> Self {
        let mesh = crate::velocity::VelocityMesh::from_field(velocity, stride);
        Self::new(device, &mesh.vertices)
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}
