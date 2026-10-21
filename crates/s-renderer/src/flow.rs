#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlowVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl FlowVertex {
    pub const CYAN: [f32; 3] = [0.0, 0.9, 1.0];
    pub const CONTOUR: [f32; 3] = [0.72, 0.82, 0.86];
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FlowVertex>() as u64,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &[
                wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { offset: 12, shader_location: 1, format: wgpu::VertexFormat::Float32x3 },
            ],
        }
    }
}
