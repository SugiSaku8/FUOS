use wgpu::util::DeviceExt;

/// GPU用海流ラインメッシュ
///
/// LineListとして描画する。
///
/// vertices:
/// [x,y,z]
///
pub struct GpuFlowMesh {
    vertex_buffer: wgpu::Buffer,

    vertex_count: u32,
}

impl GpuFlowMesh {
    pub fn new(device: &wgpu::Device, vertices: &[[f32; 3]]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("flow_vertex_buffer"),

            contents: bytemuck::cast_slice(vertices),

            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,

            vertex_count: vertices.len() as u32,
        }
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}
