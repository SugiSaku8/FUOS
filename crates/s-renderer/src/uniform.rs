use bytemuck::{Pod, Zeroable};
use nalgebra::Matrix4;
use wgpu::util::DeviceExt;

/// Camera uniform sent to the GPU.
///
/// Matrices are stored in column-major order, matching nalgebra's
/// internal representation and WGSL's `mat4x4<f32>`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    #[must_use]
    pub fn new() -> Self {
        Self {
            view: Self::identity(),
            projection: Self::identity(),
            view_projection: Self::identity(),
        }
    }

    pub fn update(&mut self, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.view = Self::to_array(view);
        self.projection = Self::to_array(projection);
        self.view_projection = Self::to_array(&(projection * view));
    }

    fn identity() -> [[f32; 4]; 4] {
        Self::to_array(&Matrix4::identity())
    }

    fn to_array(matrix: &Matrix4<f32>) -> [[f32; 4]; 4] {
        [
            [
                matrix[(0, 0)],
                matrix[(1, 0)],
                matrix[(2, 0)],
                matrix[(3, 0)],
            ],
            [
                matrix[(0, 1)],
                matrix[(1, 1)],
                matrix[(2, 1)],
                matrix[(3, 1)],
            ],
            [
                matrix[(0, 2)],
                matrix[(1, 2)],
                matrix[(2, 2)],
                matrix[(3, 2)],
            ],
            [
                matrix[(0, 3)],
                matrix[(1, 3)],
                matrix[(2, 3)],
                matrix[(3, 3)],
            ],
        ]
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

/// Owns the GPU resources associated with the camera uniform.
#[derive(Debug)]
pub struct CameraBuffer {
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = CameraUniform::default();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform_buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, view: &Matrix4<f32>, projection: &Matrix4<f32>) {
        self.uniform.update(view, projection);

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.uniform));
    }

    #[must_use]
    pub fn uniform(&self) -> &CameraUniform {
        &self.uniform
    }

    #[must_use]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    #[must_use]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[must_use]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
