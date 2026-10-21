//! Pressure gradient correction operator.

use crate::{GpuField3D, GpuVelocityField, MetalBuffer, MetalContext};

use metal::MTLSize;

pub struct GradientPressureOperator {
    pipeline: metal::ComputePipelineState,
}

impl GradientPressureOperator {
    pub fn new(context: &MetalContext) -> Self {
        let library = context.load_library(include_str!("../shaders/gradient_pressure.metal"));

        let function = library
            .get_function("gradient_pressure", None)
            .expect("gradient pressure kernel not found");

        let pipeline = context
            .device()
            .new_compute_pipeline_state_with_function(&function)
            .expect("failed to create gradient pressure pipeline");

        Self { pipeline }
    }

    /// Compatibility wrapper for callers using a constant density.
    pub fn compute(&self, context: &MetalContext, velocity: &GpuVelocityField,
        pressure: &GpuField3D, dt: f32, rho: f32) -> GpuVelocityField {
        assert!(rho.is_finite() && rho > 0.0, "rho must be positive and finite");
        self.compute_variable_density(context, velocity, pressure, dt,
            &vec![rho; velocity.grid().count()], &vec![0.0; velocity.grid().count()])
    }

    pub fn compute_variable_density(
        &self,
        context: &MetalContext,
        velocity: &GpuVelocityField,
        pressure: &GpuField3D,
        dt: f32,
        density: &[f32],
        land_mask: &[f32],
    ) -> GpuVelocityField {
        let grid = velocity.grid();
        assert_eq!(density.len(), grid.count(), "density field must match velocity grid");
        assert_eq!(land_mask.len(), grid.count(), "land mask must match velocity grid");

        let output = GpuVelocityField::empty(context, grid);

        let command_buffer = context.command_queue().new_command_buffer();

        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&self.pipeline);

        //
        // pressure
        //
        encoder.set_buffer(0, Some(pressure.buffer()), 0);

        //
        // velocity input
        //
        encoder.set_buffer(1, Some(velocity.u.buffer()), 0);

        encoder.set_buffer(2, Some(velocity.v.buffer()), 0);

        encoder.set_buffer(3, Some(velocity.w.buffer()), 0);

        //
        // velocity output
        //
        encoder.set_buffer(4, Some(output.u.buffer()), 0);

        encoder.set_buffer(5, Some(output.v.buffer()), 0);

        encoder.set_buffer(6, Some(output.w.buffer()), 0);

        //
        // GridInfo
        //
        let grid_buffer = MetalBuffer::from_struct(context, &grid);

        encoder.set_buffer(7, Some(grid_buffer.buffer()), 0);

        //
        // dt
        //
        let dt_buffer = MetalBuffer::from_f32(context.device(), &[dt]);

        encoder.set_buffer(8, Some(dt_buffer.buffer()), 0);

        //
        // Variable density at cell centres. The shader uses face-averaged inverse density.
        let density_buffer = MetalBuffer::from_f32(context.device(), density);

        encoder.set_buffer(9, Some(density_buffer.buffer()), 0);
        let mask_buffer = MetalBuffer::from_f32(context.device(), land_mask);
        encoder.set_buffer(10, Some(mask_buffer.buffer()), 0);

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
