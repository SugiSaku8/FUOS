use nalgebra::Matrix4;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::<f32>::identity().into(),
        }
    }

    pub fn update(&mut self, matrix: Matrix4<f32>) {
        self.view_proj = matrix.into();
    }
}

pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,

    pub uniform: CameraUniform,
}

impl CameraBuffer {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> Self {
        let uniform = CameraUniform::new();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),

            contents: bytemuck::bytes_of(&uniform),

            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),

            layout,

            entries: &[wgpu::BindGroupEntry {
                binding: 0,

                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,

            bind_group,

            uniform,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.uniform));
    }
}
