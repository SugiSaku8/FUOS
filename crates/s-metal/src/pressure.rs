//! GPU variable-density pressure projection.
use crate::{GpuField3D, MetalBuffer, MetalContext};
use metal::MTLSize;

pub struct PressureSolver { pipeline: metal::ComputePipelineState }
impl PressureSolver {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/pressure.metal"));
        let function = library.get_function("pressure_jacobi", None).expect("pressure kernel not found");
        let pipeline = context.device().new_compute_pipeline_state_with_function(&function)
            .expect("failed to create pressure pipeline");
        Self { pipeline }
    }

    /// Compatibility wrapper for examples using a constant density.
    pub fn solve(&self, context: &MetalContext, divergence: &GpuField3D,
        dt: f32, rho: f32, iterations: usize) -> GpuField3D {
        assert!(rho.is_finite() && rho > 0.0, "rho must be positive and finite");
        self.solve_variable_density(context, divergence, dt, &vec![rho; divergence.len()], iterations)
    }

    /// Variable-density solve without land cells.
    pub fn solve_variable_density(&self, context: &MetalContext, divergence: &GpuField3D,
        dt: f32, density: &[f32], iterations: usize) -> GpuField3D {
        self.solve_variable_density_masked(context, divergence, dt, density,
            &vec![0.0; divergence.len()], iterations)
    }

    /// Solve div((1/rho) grad(p)) = div(u*)/dt using variable density and a land mask.
    /// All Jacobi passes are encoded into one command buffer to avoid per-iteration CPU/GPU waits.
    pub fn solve_variable_density_masked(
        &self, context: &MetalContext, divergence: &GpuField3D,
        dt: f32, density: &[f32], land_mask: &[f32], iterations: usize,
    ) -> GpuField3D {
        assert!(dt.is_finite() && dt > 0.0, "dt must be positive and finite");
        assert_eq!(density.len(), divergence.len(), "density and divergence sizes must match");
        assert_eq!(land_mask.len(), divergence.len(), "land mask and divergence sizes must match");
        let grid = divergence.grid();
        let mut pressure = GpuField3D::empty(context, grid);
        let mut next = GpuField3D::empty(context, grid);
        let grid_buffer = MetalBuffer::from_struct(context, &grid);
        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);
        let density_buffer = MetalBuffer::from_f32(context.device(), density);
        let mask_buffer = MetalBuffer::from_f32(context.device(), land_mask);
        let gauge_index = land_mask.iter().position(|value| *value <= 0.5).unwrap_or(0) as u32;
        let gauge_buffer = MetalBuffer::from_struct(context, &gauge_index);
        let threads = MTLSize { width: 8, height: 8, depth: 8 };
        let groups = MTLSize {
            width: ((grid.nx()+7)/8) as u64,
            height: ((grid.ny()+7)/8) as u64,
            depth: ((grid.nz()+7)/8) as u64,
        };
        let command_buffer = context.command_queue().new_command_buffer();
        for _ in 0..iterations {
            let encoder = command_buffer.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(&self.pipeline);
            encoder.set_buffer(0, Some(pressure.buffer()), 0);
            encoder.set_buffer(1, Some(divergence.buffer()), 0);
            encoder.set_buffer(2, Some(next.buffer()), 0);
            encoder.set_buffer(3, Some(grid_buffer.buffer()), 0);
            encoder.set_buffer(4, Some(dt_buffer.buffer()), 0);
            encoder.set_buffer(5, Some(density_buffer.buffer()), 0);
            encoder.set_buffer(6, Some(mask_buffer.buffer()), 0);
            encoder.set_buffer(7, Some(gauge_buffer.buffer()), 0);
            encoder.dispatch_thread_groups(groups, threads);
            encoder.end_encoding();
            std::mem::swap(&mut pressure, &mut next);
        }
        command_buffer.commit();
        command_buffer.wait_until_completed();
        pressure
    }
}
