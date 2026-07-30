#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlowVertex {
    pub position: [f32; 3],
}

impl FlowVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FlowVertex>() as u64,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &[wgpu::VertexAttribute {
                offset: 0,

                shader_location: 0,

                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
