//! Land mask operator.

use crate::{GpuField3D, GpuVelocityField, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct LandMaskOperator {
    pipeline: metal::ComputePipelineState,
}

impl LandMaskOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/land_mask.metal"));

        let function = library
            .get_function("land_mask_step", None)
            .expect("land mask kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create land mask pipeline");

        Self { pipeline }
    }

    pub fn apply(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
        mask: &GpuField3D,
    ) -> GpuVelocityField {
        let grid = velocity.grid();

        let output = GpuVelocityField::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        encoder.set_buffer(0, Some(mask.buffer()), 0);

        encoder.set_buffer(1, Some(velocity.u.buffer()), 0);
        encoder.set_buffer(2, Some(velocity.v.buffer()), 0);
        encoder.set_buffer(3, Some(velocity.w.buffer()), 0);

        encoder.set_buffer(4, Some(output.u.buffer()), 0);
        encoder.set_buffer(5, Some(output.v.buffer()), 0);
        encoder.set_buffer(6, Some(output.w.buffer()), 0);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(7, Some(grid_buffer.buffer()), 0);

        let threads = MTLSize {
            width: 8,
            height: 8,
            depth: 8,
        };

        let groups = MTLSize {
            width: ((grid.nx() + 7) / 8) as u64,
            height: ((grid.ny() + 7) / 8) as u64,
            depth: ((grid.nz() + 7) / 8) as u64,
        };

        encoder.dispatch_thread_groups(groups, threads);

        encoder.end_encoding();

        command_buffer.commit();

        command_buffer.wait_until_completed();

        output
    }
}
