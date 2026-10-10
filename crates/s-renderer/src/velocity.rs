use s_field::VelocityField;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VelocityVertex {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
}

impl VelocityVertex {
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VelocityVertex>() as wgpu::BufferAddress,

            step_mode: wgpu::VertexStepMode::Vertex,

            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<VelocityVertex>() as wgpu::BufferAddress,

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
            ],
        }
    }
}

pub struct VelocityMesh {
    pub vertices: Vec<VelocityVertex>,
    pub indices: Vec<u32>,
}

impl VelocityMesh {
    pub fn from_field(field: &VelocityField, stride: usize) -> Self {
        let shape = field.shape();

        let nx = shape[2];
        let ny = shape[1];
        let nz = shape[0];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut id = 0u32;

        for z in (0..nz).step_by(stride) {
            for y in (0..ny).step_by(stride) {
                for x in (0..nx).step_by(stride) {
                    let u = *field.u().get(0, z, y, x).unwrap_or(&0.0);

                    let v = *field.v().get(0, z, y, x).unwrap_or(&0.0);

                    let w = *field.w().get(0, z, y, x).unwrap_or(&0.0);

                    let scale = 20.0;

                    vertices.push(VelocityVertex {
                        position: [x as f32, y as f32, -(z as f32)],
                        velocity: [u, v, w],
                    });

                    vertices.push(VelocityVertex {
                        position: [
                            x as f32 + u * scale,
                            y as f32 + v * scale,
                            -(z as f32) + w * scale,
                        ],
                        velocity: [u, v, w],
                    });

                    indices.push(id);
                    indices.push(id + 1);

                    id += 2;
                }
            }
        }

        Self { vertices, indices }
    }
}
