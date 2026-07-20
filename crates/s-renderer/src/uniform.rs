use bytemuck::{Pod, Zeroable};

use nalgebra::Matrix4;

use crate::buffer::UniformBuffer;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],

    pub projection: [[f32; 4]; 4],

    pub view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view: Matrix4::<f32>::identity().into(),

            projection: Matrix4::<f32>::identity().into(),

            view_projection: Matrix4::<f32>::identity().into(),
        }
    }

    pub fn update(&mut self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.view = (*view).into();

        self.projection = (*projection).into();

        self.view_projection = (*projection * *view).into();
    }
}

#[derive(Debug)]
pub struct CameraBuffer {
    buffer: UniformBuffer<CameraUniform>,

    bind_group: wgpu::BindGroup,

    layout: wgpu::BindGroupLayout,
}

impl CameraBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = CameraUniform::new();

        let buffer = UniformBuffer::new(device, uniform, "camera_uniform");

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),

            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,

                visibility: wgpu::ShaderStages::VERTEX,

                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,

                    has_dynamic_offset: false,

                    min_binding_size: None,
                },

                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),

            layout: &layout,

            entries: &[wgpu::BindGroupEntry {
                binding: 0,

                resource: buffer.buffer().as_entire_binding(),
            }],
        });

        Self {
            buffer,

            bind_group,

            layout,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        let mut uniform = CameraUniform::new();

        uniform.update(view, projection);

        self.buffer.update(queue, uniform);
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    #[inline]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}
