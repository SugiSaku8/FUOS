//! GPU divergence operator.

use crate::{GpuField3D, GpuVelocityField, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct DivergenceOperator {
    pipeline: metal::ComputePipelineState,
}

impl DivergenceOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/divergence.metal"));

        let function = library
            .get_function("divergence_step", None)
            .expect("divergence kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create divergence pipeline");

        Self { pipeline }
    }

    pub fn compute(&self, context: &MetalContext, velocity: &GpuVelocityField) -> GpuField3D {
        let grid = velocity.grid();

        let output = GpuField3D::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        encoder.set_buffer(0, Some(velocity.u.buffer()), 0);

        encoder.set_buffer(1, Some(velocity.v.buffer()), 0);

        encoder.set_buffer(2, Some(velocity.w.buffer()), 0);

        encoder.set_buffer(3, Some(output.buffer()), 0);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(4, Some(grid_buffer.buffer()), 0);

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
