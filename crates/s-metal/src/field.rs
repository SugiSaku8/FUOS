use crate::{GridInfo, MetalBuffer, MetalContext};

pub struct GpuField3D {
    pub data: MetalBuffer,

    pub grid: GridInfo,
}

impl GpuField3D {
    pub fn from_f32(context: &MetalContext, data: &[f32], grid: GridInfo) -> Self {
        Self {
            data: MetalBuffer::from_f32(context, data),

            grid,
        }
    }

    pub fn empty(context: &MetalContext, grid: GridInfo) -> Self {
        let data = vec![0.0f32; grid.count()];

        Self::from_f32(context, &data, grid)
    }

    pub fn buffer(&self) -> &metal::Buffer {
        self.data.raw()
    }

    pub fn grid(&self) -> GridInfo {
        self.grid
    }
}
