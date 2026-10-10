use wgpu::util::DeviceExt;

use crate::flow::FlowVertex;

/// GPU上に保持する潮流線メッシュ。
pub struct GpuFlowMesh {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    capacity_bytes: u64,
}

impl GpuFlowMesh {
    /// 指定した最大頂点数でGPUバッファを確保する。
    pub fn with_capacity(device: &wgpu::Device, max_vertices: usize) -> Self {
        let capacity_vertices = max_vertices.max(1);

        let capacity_bytes = (capacity_vertices * std::mem::size_of::<FlowVertex>()) as u64;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dynamic_flow_vertex_buffer"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            vertex_count: 0,
            capacity_bytes,
        }
    }

    /// 初期頂点データからGPUメッシュを作成する。
    pub fn from_flow_vertices(device: &wgpu::Device, vertices: &[FlowVertex]) -> Self {
        let capacity_vertices = vertices.len().max(1);

        let capacity_bytes = (capacity_vertices * std::mem::size_of::<FlowVertex>()) as u64;

        let initial_contents = if vertices.is_empty() {
            bytemuck::cast_slice(&[FlowVertex {
                position: [0.0, 0.0, 0.0],
            }])
        } else {
            bytemuck::cast_slice(vertices)
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("flow_vertex_buffer"),
            contents: initial_contents,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            capacity_bytes,
        }
    }

    /// CPU側で更新した頂点データをGPUへ転送する。
    ///
    /// バッファの容量を超えた場合は false を返す。
    pub fn update_vertices(&mut self, queue: &wgpu::Queue, vertices: &[FlowVertex]) -> bool {
        let required_bytes = (vertices.len() * std::mem::size_of::<FlowVertex>()) as u64;

        if required_bytes > self.capacity_bytes {
            return false;
        }

        if !vertices.is_empty() {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(vertices));
        }

        self.vertex_count = vertices.len() as u32;

        true
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}
