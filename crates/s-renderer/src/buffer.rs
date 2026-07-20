use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// Generic GPU buffer wrapper.
///
/// Used for:
/// - Vertex buffer
/// - Index buffer
/// - Uniform buffer
/// - Storage buffer
#[derive(Debug)]
pub struct GpuBuffer {
    buffer: wgpu::Buffer,
    size: u64,
}

impl GpuBuffer {
    /// Creates a GPU buffer from typed data.
    pub fn from_data<T>(
        device: &wgpu::Device,
        data: &[T],
        usage: wgpu::BufferUsages,
        label: &'static str,
    ) -> Self
    where
        T: Pod,
    {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),

            contents: bytemuck::cast_slice(data),

            usage,
        });

        Self {
            buffer,

            size: std::mem::size_of_val(data) as u64,
        }
    }

    /// Creates an empty GPU buffer.
    pub fn empty(
        device: &wgpu::Device,
        size: u64,
        usage: wgpu::BufferUsages,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),

            size,

            usage,

            mapped_at_creation: false,
        });

        Self { buffer, size }
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
}

/// Uniform buffer.
///
/// Intended for:
/// - Camera
/// - Material
/// - Simulation parameters
#[derive(Debug)]
pub struct UniformBuffer<T>
where
    T: Pod + Zeroable,
{
    buffer: GpuBuffer,
    value: T,
}

impl<T> UniformBuffer<T>
where
    T: Pod + Zeroable + Copy,
{
    pub fn new(device: &wgpu::Device, value: T, label: &'static str) -> Self {
        let buffer = GpuBuffer::from_data(
            device,
            &[value],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            label,
        );

        Self { buffer, value }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, value: T) {
        self.value = value;

        queue.write_buffer(self.buffer.buffer(), 0, bytemuck::bytes_of(&value));
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.buffer()
    }
}

/// Storage buffer.
///
/// Used for GPU compute.
///
/// Example:
/// Ocean velocity field
/// Temperature field
/// Salinity field
#[derive(Debug)]
pub struct StorageBuffer<T>
where
    T: Pod,
{
    buffer: GpuBuffer,

    length: usize,

    marker: std::marker::PhantomData<T>,
}

impl<T> StorageBuffer<T>
where
    T: Pod,
{
    pub fn from_data(
        device: &wgpu::Device,
        data: &[T],
        writable: bool,
        label: &'static str,
    ) -> Self {
        let mut usage = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC;

        if writable {
            usage |= wgpu::BufferUsages::MAP_READ;
        }

        let buffer = GpuBuffer::from_data(device, data, usage, label);

        Self {
            buffer,
            length: data.len(),

            marker: std::marker::PhantomData,
        }
    }

    #[inline]
    pub fn buffer(&self) -> &wgpu::Buffer {
        self.buffer.buffer()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }
}
