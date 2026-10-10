//! GPU field representation.
use crate::{GridInfo, MetalBuffer, MetalContext};
use metal::NSRange;

pub struct GpuField3D {
    pub data: MetalBuffer,

    grid: GridInfo,
}

impl GpuField3D {
    pub fn from_f32(context: &MetalContext, values: &[f32], grid: GridInfo) -> Self {
        Self {
            data: MetalBuffer::from_f32(context.device(), values),

            grid,
        }
    }

    pub fn empty(context: &MetalContext, grid: GridInfo) -> Self {
        let zero = vec![0.0f32; grid.count()];

        Self {
            data: MetalBuffer::from_f32(context.device(), &zero),

            grid,
        }
    }

    #[inline]
    pub fn buffer(&self) -> &metal::Buffer {
        self.data.raw()
    }

    #[inline]
    pub fn grid(&self) -> GridInfo {
        self.grid
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.grid.count()
    }

    pub fn download(&self, context: &MetalContext) -> Vec<f32> {
        //
        // Wait for GPU completion.
        //
        // (context is currently unused,
        // but kept for API consistency)
        //
        let _ = context;

        let buffer = self.data.raw();

        //
        // Synchronize shared memory
        //
        buffer.did_modify_range(NSRange {
            location: 0,

            length: buffer.length(),
        });
        let ptr = buffer.contents() as *const f32;

        unsafe { std::slice::from_raw_parts(ptr, self.len()).to_vec() }
    }

    pub fn upload(&self, _context: &MetalContext, values: &[f32]) {
        let buffer = self.data.raw();

        unsafe {
            let ptr = buffer.contents() as *mut f32;

            std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len());
        }
    }
}
