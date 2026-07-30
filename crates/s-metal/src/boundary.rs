//! GPU boundary condition operator.

use crate::{GpuVelocityField, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct BoundaryOperator {
    pipeline: metal::ComputePipelineState,
}

impl BoundaryOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/boundary.metal"));

        let function = library
            .get_function("boundary_step", None)
            .expect("boundary kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create boundary pipeline");

        Self { pipeline }
    }

    pub fn apply(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
        inflow_u: f32,
        inflow_v: f32,
        inflow_w: f32,
    ) -> GpuVelocityField {
        let grid = velocity.grid();

        let output = GpuVelocityField::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        //
        // input
        //
        encoder.set_buffer(0, Some(velocity.u.buffer()), 0);
        encoder.set_buffer(1, Some(velocity.v.buffer()), 0);
        encoder.set_buffer(2, Some(velocity.w.buffer()), 0);

        //
        // output
        //
        encoder.set_buffer(3, Some(output.u.buffer()), 0);
        encoder.set_buffer(4, Some(output.v.buffer()), 0);
        encoder.set_buffer(5, Some(output.w.buffer()), 0);

        //
        // grid
        //
        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(6, Some(grid_buffer.buffer()), 0);

        //
        // inflow velocity
        //
        let u_buffer = MetalBuffer::from_f32(context.device(), &[inflow_u]);
        let v_buffer = MetalBuffer::from_f32(context.device(), &[inflow_v]);
        let w_buffer = MetalBuffer::from_f32(context.device(), &[inflow_w]);

        encoder.set_buffer(7, Some(u_buffer.buffer()), 0);
        encoder.set_buffer(8, Some(v_buffer.buffer()), 0);
        encoder.set_buffer(9, Some(w_buffer.buffer()), 0);

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
