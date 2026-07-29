use metal::{Buffer, MTLResourceOptions};

use crate::context::MetalContext;

use std::mem;

pub struct MetalBuffer {
    buffer: Buffer,

    length: usize,
}

impl MetalBuffer {
    /// Vec<f32>からGPU Bufferを生成
    pub fn from_f32(context: &MetalContext, data: &[f32]) -> Self {
        let size = std::mem::size_of_val(data);

        let buffer = context.device().new_buffer_with_data(
            data.as_ptr() as *const _,
            size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Self {
            buffer,

            length: data.len(),
        }
    }

    /// 空のGPU Bufferを生成
    pub fn empty(context: &MetalContext, length: usize) -> Self {
        let size = (length * mem::size_of::<f32>()) as u64;

        let buffer = context
            .device()
            .new_buffer(size, MTLResourceOptions::StorageModeShared);

        Self { buffer, length }
    }

    /// 任意の構造体をGPU Buffer化
    ///
    /// GridInfoなど
    /// #[repr(C)]構造体用
    pub fn from_struct<T>(context: &MetalContext, value: &T) -> Self {
        let size = mem::size_of::<T>();

        let buffer = context.device().new_buffer_with_data(
            value as *const T as *const _,
            size as u64,
            MTLResourceOptions::StorageModeShared,
        );

        Self {
            buffer,

            // f32単位として扱う
            length: (size + 3) / 4,
        }
    }

    /// Metal Buffer取得
    pub fn raw(&self) -> &Buffer {
        &self.buffer
    }

    /// GPU BufferをVec<f32>へコピー
    pub fn read_f32(&self) -> Vec<f32> {
        let ptr = self.buffer.contents() as *const f32;

        let slice = unsafe { std::slice::from_raw_parts(ptr, self.length) };

        slice.to_vec()
    }
}
