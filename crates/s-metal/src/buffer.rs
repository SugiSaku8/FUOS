//! Metal buffer wrapper.

use metal::{Buffer, Device, MTLResourceOptions};

use crate::MetalContext;

pub struct MetalBuffer {
    buffer: Buffer,
}

impl MetalBuffer {
    /// Wrap existing Metal buffer.
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer }
    }

    /// Create buffer from f32 slice.
    pub fn from_f32(device: &Device, data: &[f32]) -> Self {
        let buffer = device.new_buffer_with_data(
            data.as_ptr() as *const std::ffi::c_void,
            (data.len() * std::mem::size_of::<f32>()) as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Self { buffer }
    }

    /// Create empty GPU buffer.
    ///
    /// Used as compute shader output.
    pub fn empty(device: &Device, count: usize) -> Self {
        let size = count * std::mem::size_of::<f32>();

        let buffer = device.new_buffer(size as u64, MTLResourceOptions::StorageModeShared);

        Self { buffer }
    }

    /// Create buffer from small uniform struct.
    ///
    /// Used for:
    ///
    /// - GridInfo
    /// - timestep
    /// - constants
    ///
    pub fn from_struct<T>(context: &MetalContext, value: &T) -> Self {
        let size = std::mem::size_of::<T>();

        let buffer = context.device().new_buffer_with_data(
            value as *const T as *const std::ffi::c_void,
            size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Self { buffer }
    }

    /// Access raw Metal buffer.
    #[inline]
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Mutable raw access.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    /// GPU pointer.
    #[inline]
    pub fn contents(&self) -> *mut std::ffi::c_void {
        self.buffer.contents()
    }

    /// Buffer length in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        self.buffer.length() as usize
    }

    #[inline]

    pub fn raw(&self) -> &Buffer {
        &self.buffer
    }
}
