use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// CPU side geometry.
///
/// This mirrors s-core Geometry.
/// Renderer does not know the actual vertex type.
#[derive(Debug, Clone)]
pub struct Mesh<V>
where
    V: Pod,
{
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

impl<V> Mesh<V>
where
    V: Pod,
{
    #[must_use]
    pub fn new(vertices: Vec<V>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    #[inline]
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

/// GPU uploaded mesh.
///
/// Generic over vertex format.
#[derive(Debug)]
pub struct GpuMesh<V>
where
    V: Pod,
{
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    index_count: u32,

    marker: std::marker::PhantomData<V>,
}

impl<V> GpuMesh<V>
where
    V: Pod,
{
    pub fn upload(device: &wgpu::Device, mesh: &Mesh<V>) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_vertex_buffer"),

            contents: bytemuck::cast_slice(&mesh.vertices),

            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh_index_buffer"),

            contents: bytemuck::cast_slice(&mesh.indices),

            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,

            index_count: mesh.indices.len() as u32,

            marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    #[inline]
    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    #[inline]
    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    /// Bind mesh buffers to render pass.
    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    /// Draw mesh.
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
