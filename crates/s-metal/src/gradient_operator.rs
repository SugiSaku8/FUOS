use crate::{GpuField3D, Gradient3D, GridInfo, MetalBuffer, MetalContext};

use crate::compute::dispatch_with_grid;

use metal::ComputePipelineState;

pub struct GradientOperator {
    dx_pipeline: ComputePipelineState,

    dy_pipeline: ComputePipelineState,

    dz_pipeline: ComputePipelineState,
}

impl GradientOperator {
    pub fn new(
        dx_pipeline: ComputePipelineState,
        dy_pipeline: ComputePipelineState,
        dz_pipeline: ComputePipelineState,
    ) -> Self {
        Self {
            dx_pipeline,

            dy_pipeline,

            dz_pipeline,
        }
    }

    fn derivative(
        &self,
        pipeline: &ComputePipelineState,

        context: &MetalContext,

        input: &GpuField3D,

        grid: GridInfo,
    ) -> GpuField3D {
        let output = GpuField3D::empty(context, grid);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        dispatch_with_grid(
            pipeline,
            context.queue(),
            input.buffer(),
            output.buffer(),
            grid_buffer.raw(),
            grid.count(),
        );

        output
    }

    pub fn gradient(&self, context: &MetalContext, field: &GpuField3D) -> Gradient3D {
        let grid = field.grid();

        let dx = self.derivative(&self.dx_pipeline, context, field, grid);

        let dy = self.derivative(&self.dy_pipeline, context, field, grid);

        let dz = self.derivative(&self.dz_pipeline, context, field, grid);

        Gradient3D { dx, dy, dz }
    }
}
