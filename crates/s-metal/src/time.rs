use crate::{GpuField3D, MetalBuffer, MetalContext};

use crate::compute::dispatch_custom;

use metal::ComputePipelineState;

pub struct EulerIntegrator {
    pipeline: ComputePipelineState,
}

impl EulerIntegrator {
    pub fn new(pipeline: ComputePipelineState) -> Self {
        Self { pipeline }
    }

    pub fn step(
        &self,

        context: &MetalContext,

        current: &GpuField3D,

        derivative: &GpuField3D,

        dt: f32,
    ) -> GpuField3D {
        let grid = current.grid();

        let output = GpuField3D::empty(context, grid);

        let dt_buffer = MetalBuffer::from_struct(context, &dt);

        dispatch_custom(
            &self.pipeline,
            context.queue(),
            &[
                current.buffer(),
                derivative.buffer(),
                output.buffer(),
                dt_buffer.raw(),
            ],
            grid.count(),
        );

        output
    }
}
