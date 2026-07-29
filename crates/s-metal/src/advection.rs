//! GPU advection operator.

use crate::{GpuVelocityField, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct AdvectionOperator {
    pipeline: metal::ComputePipelineState,
}

impl AdvectionOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/advection.metal"));

        let function = library
            .get_function("advection_step", None)
            .expect("advection kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create advection pipeline");

        Self { pipeline }
    }

    pub fn compute(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
        dt: f32,
    ) -> GpuVelocityField {
        //
        // output field
        //
        let grid = velocity.grid();

        let output = GpuVelocityField::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        //
        // input velocity
        //

        encoder.set_buffer(0, Some(velocity.u.buffer()), 0);

        encoder.set_buffer(1, Some(velocity.v.buffer()), 0);

        encoder.set_buffer(2, Some(velocity.w.buffer()), 0);

        //
        // output velocity
        //

        encoder.set_buffer(3, Some(output.u.buffer()), 0);

        encoder.set_buffer(4, Some(output.v.buffer()), 0);

        encoder.set_buffer(5, Some(output.w.buffer()), 0);

        //
        // GridInfo
        //

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(6, Some(grid_buffer.raw()), 0);

        //
        // timestep
        //

        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);
        encoder.set_buffer(7, Some(dt_buffer.raw()), 0);

        //
        // dispatch
        //

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
