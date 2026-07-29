//! GPU pressure solver.
//!
//! Solves:
//!
//! ∇²p = divergence * rho / dt
//!
//! using Jacobi iteration.

use crate::{GpuField3D, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct PressureSolver {
    pipeline: metal::ComputePipelineState,
}

impl PressureSolver {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/pressure.metal"));

        let function = library
            .get_function("pressure_jacobi", None)
            .expect("pressure kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create pressure pipeline");

        Self { pipeline }
    }

    pub fn solve(
        &self,

        context: &MetalContext,

        divergence: &GpuField3D,

        iterations: usize,
    ) -> GpuField3D {
        let grid = divergence.grid();

        let mut pressure = GpuField3D::empty(context, grid);

        let mut next = GpuField3D::empty(context, grid);

        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        for _ in 0..iterations {
            let command_buffer = context.command_queue().new_command_buffer();

            let encoder = command_buffer.new_compute_command_encoder();

            encoder.set_compute_pipeline_state(&self.pipeline);

            //
            // input pressure
            //

            encoder.set_buffer(0, Some(pressure.buffer()), 0);

            //
            // divergence
            //

            encoder.set_buffer(1, Some(divergence.buffer()), 0);

            //
            // output
            //

            encoder.set_buffer(2, Some(next.buffer()), 0);

            //
            // grid
            //

            encoder.set_buffer(3, Some(grid_buffer.buffer()), 0);

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

            std::mem::swap(&mut pressure, &mut next);
        }

        pressure
    }
}
